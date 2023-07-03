import { Buffer } from 'buffer';
import { v4 as uuid } from 'uuid';
import { assert, Optional } from '@votingworks/basics';
import { Client as DbClient } from '@votingworks/db';
import {
  BallotStyleId,
  ElectionDefinition,
  Id,
  PrecinctId,
  safeParseElectionDefinition,
  SystemSettings,
  SystemSettingsDbRow,
  VotesDict,
} from '@votingworks/types';
import { join } from 'path';
import { DateTime } from 'luxon';
import {
  VoterElectionRegistration,
  VoterInfo,
  VoterRegistrationRequest,
} from './types/db';

const SchemaPath = join(__dirname, '../schema.sql');

/**
 * Manages a data store for imported election definition and system settings
 */
export class Store {
  private constructor(private readonly client: DbClient) {}

  getDbPath(): string {
    return this.client.getDatabasePath();
  }

  /**
   * Builds and returns a new store whose data is kept in memory.
   */
  static memoryStore(): Store {
    return new Store(DbClient.memoryClient(SchemaPath));
  }

  /**
   * Builds and returns a new store at `dbPath`.
   */
  static fileStore(dbPath: string): Store {
    return new Store(DbClient.fileClient(dbPath, SchemaPath));
  }

  /**
   * Resets the database and any cached data in the store.
   */
  reset(): void {
    this.client.reset();
  }

  /**
   * Gets the current election definition.
   */
  getElectionDefinition(id: Id): Optional<ElectionDefinition> {
    const electionRow = this.client.one(
      `select
        election_data as electionData
      from elections
      where id = ?
      `,
      id
    ) as Optional<{ electionData: string }>;

    if (!electionRow?.electionData) {
      return undefined;
    }

    return safeParseElectionDefinition(electionRow.electionData).assertOk(
      'Unable to parse stored election data.'
    );
  }

  /**
   * Creates a new election definition from a JSON string.
   */
  createElectionDefinition(electionData: string): Id {
    safeParseElectionDefinition(electionData).assertOk(
      `Unable to parse election data: ${electionData}`
    );

    const id = uuid();
    this.client.run(
      `
      insert into elections (
        id,
        election_data
      ) values (
        ?, ?
      )
      `,
      id,
      electionData
    );
    return id;
  }

  /**
   * Deletes system settings
   */
  deleteSystemSettings(): void {
    this.client.run('delete from system_settings');
  }

  /**
   * Creates a system settings record
   */
  setSystemSettings(systemSettings: SystemSettings): void {
    this.client.run('delete from system_settings');
    this.client.run(
      `
      insert into system_settings (
        are_poll_worker_card_pins_enabled,
        inactive_session_time_limit_minutes,
        num_incorrect_pin_attempts_allowed_before_card_lockout,
        overall_session_time_limit_hours,
        starting_card_lockout_duration_seconds
      ) values (
        ?, ?, ?, ?, ?
      )
      `,
      systemSettings.arePollWorkerCardPinsEnabled ? 1 : 0,
      systemSettings.inactiveSessionTimeLimitMinutes,
      systemSettings.numIncorrectPinAttemptsAllowedBeforeCardLockout,
      systemSettings.overallSessionTimeLimitHours,
      systemSettings.startingCardLockoutDurationSeconds
    );
  }

  /**
   * Gets system settings or undefined if they aren't loaded yet
   */
  getSystemSettings(): SystemSettings | undefined {
    const result = this.client.one(
      `
      select
        are_poll_worker_card_pins_enabled as arePollWorkerCardPinsEnabled,
        inactive_session_time_limit_minutes as inactiveSessionTimeLimitMinutes,
        num_incorrect_pin_attempts_allowed_before_card_lockout as numIncorrectPinAttemptsAllowedBeforeCardLockout,
        overall_session_time_limit_hours as overallSessionTimeLimitHours,
        starting_card_lockout_duration_seconds as startingCardLockoutDurationSeconds
      from system_settings
      `
    ) as SystemSettingsDbRow | undefined;

    if (!result) {
      return undefined;
    }
    return {
      ...result,
      arePollWorkerCardPinsEnabled: result.arePollWorkerCardPinsEnabled === 1,
    };
  }

  /**
   * Gets basic information about a voter by CAC ID.
   */
  getVoterInfo(commonAccessCardId: Id): Optional<VoterInfo> {
    const result = this.client.one(
      `
      select
        id,
        common_access_card_id as commonAccessCardId,
        is_admin as isAdmin
      from voters
      where common_access_card_id = ?
      `,
      commonAccessCardId
    ) as Optional<{ id: string; commonAccessCardId: string; isAdmin: 0 | 1 }>;

    if (!result) {
      return undefined;
    }

    return {
      id: result.id,
      commonAccessCardId: result.commonAccessCardId,
      isAdmin: result.isAdmin === 1,
    };
  }

  /**
   * Gets or creates a voter by CAC ID.
   */
  getOrCreateVoterInfo(commonAccessCardId: Id): VoterInfo {
    const existingVoterInfo = this.getVoterInfo(commonAccessCardId);

    if (existingVoterInfo) {
      return existingVoterInfo;
    }

    const id = uuid();
    this.client.run(
      `insert into voters (id, common_access_card_id) values (?, ?)`,
      id,
      commonAccessCardId
    );

    const newVoterInfo = this.getVoterInfo(commonAccessCardId);
    assert(newVoterInfo, 'Failed to create voter info');
    return newVoterInfo;
  }

  /**
   * Sets whether a voter is an admin.
   */
  setVoterIsAdmin(id: Id, isAdmin: boolean): void {
    this.client.run(
      `
      update voters
      set is_admin = ?
      where id = ?
      `,
      isAdmin ? 1 : 0,
      id
    );
  }

  /**
   * Gets all the registrations for a given voter by database ID.
   *
   * @param voterId The database ID of the voter, notably NOT the CAC ID.
   */
  getVoterRegistrationRequests(voterId: Id): VoterRegistrationRequest[] {
    const result = this.client.all(
      `
      select
        id,
        voter_id as voterId,
        given_name as givenName,
        family_name as familyName,
        address_line_1 as addressLine1,
        address_line_2 as addressLine2,
        city,
        state,
        postal_code as postalCode,
        state_id as stateId
      from voter_registration_requests
      where voter_id = ?
      `,
      voterId
    ) as Array<{
      id: string;
      voterId: string;
      givenName: string;
      familyName: string;
      addressLine1: string;
      addressLine2: string | null;
      city: string;
      state: string;
      postalCode: string;
      stateId: string;
    }>;

    return result.map((row) => ({
      id: row.id,
      voterId: row.voterId,
      givenName: row.givenName,
      familyName: row.familyName,
      addressLine1: row.addressLine1,
      addressLine2: row.addressLine2 ?? undefined,
      city: row.city,
      state: row.state,
      postalCode: row.postalCode,
      stateId: row.stateId,
    }));
  }

  /**
   * @returns registrations sorted by most recent first
   */
  getVoterElectionRegistrations(voterId: Id): VoterElectionRegistration[] {
    const result = this.client.all(
      `
      select
        id,
        voter_id as voterId,
        voter_registration_request_id as voterRegistrationRequestId,
        election_id as electionId,
        precinct_id as precinctId,
        ballot_style_id as ballotStyleId,
        created_at as createdAt
      from voter_election_registrations
      where voter_id = ?
      order by created_at desc
      `,
      voterId
    ) as Array<{
      id: string;
      voterId: string;
      voterRegistrationRequestId: string;
      electionId: string;
      precinctId: string;
      ballotStyleId: string;
      createdAt: string;
    }>;

    return result.map((row) => ({
      id: row.id,
      voterId: row.voterId,
      voterRegistrationRequestId: row.voterRegistrationRequestId,
      electionId: row.electionId,
      precinctId: row.precinctId,
      ballotStyleId: row.ballotStyleId,
      createdAt: DateTime.fromSQL(row.createdAt),
    }));
  }

  /**
   * Gets the election for the given voter registration ID.
   */
  getElectionDefinitionForVoterRegistration(
    voterRegistrationId: Id
  ): Optional<ElectionDefinition> {
    const result = this.client.one(
      `
      select
        election_data as electionData
      from voter_election_registrations
      inner join elections on elections.id = voter_election_registrations.election_id
      where voter_election_registrations.id = ?
      `,
      voterRegistrationId
    ) as Optional<{ electionData: string | Buffer }>;

    if (!result) {
      return undefined;
    }

    const electionDefinitionParseResult = safeParseElectionDefinition(
      result.electionData.toString()
    );

    if (electionDefinitionParseResult.isErr()) {
      throw new Error('Unable to parse stored election data.');
    }

    return electionDefinitionParseResult.ok();
  }

  /**
   * Gets the votes for the given voter registration ID.
   */
  getVotesForVoterRegistration(voterRegistrationId: Id): Optional<VotesDict> {
    const result = this.client.one(
      `
      select
        votes_json as votesJson
      from voter_election_selections
      where voter_election_registration_id = ?
      `,
      voterRegistrationId
    ) as Optional<{ votesJson: string }>;

    if (!result) {
      return undefined;
    }

    return JSON.parse(result.votesJson);
  }

  /**
   * Creates a voter registration request for the voter with the given Common
   * Access Card ID.
   */
  createVoterRegistrationRequest({
    commonAccessCardId,
    givenName,
    familyName,
    addressLine1,
    addressLine2,
    city,
    state,
    postalCode,
    stateId,
  }: {
    commonAccessCardId: Id;
    givenName: string;
    familyName: string;
    addressLine1: string;
    addressLine2?: string;
    city: string;
    state: string;
    postalCode: string;
    stateId: string;
  }): Id {
    const voterInfo = this.getOrCreateVoterInfo(commonAccessCardId);
    const id = uuid();

    this.client.run(
      `
      insert into voter_registration_requests (
        id,
        voter_id,
        given_name,
        family_name,
        address_line_1,
        address_line_2,
        city,
        state,
        postal_code,
        state_id
      ) values (
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      voterInfo.id,
      givenName,
      familyName,
      addressLine1,
      addressLine2 ?? null,
      city,
      state,
      postalCode,
      stateId
    );

    return id;
  }

  /**
   * Associates a voter registration with an election.
   */
  createVoterElectionRegistration({
    voterId,
    voterRegistrationRequestId,
    electionId,
    precinctId,
    ballotStyleId,
  }: {
    voterId: Id;
    voterRegistrationRequestId: Id;
    electionId: Id;
    precinctId: PrecinctId;
    ballotStyleId: BallotStyleId;
  }): Id {
    const id = uuid();
    const voterRegistrationRequests =
      this.getVoterRegistrationRequests(voterId);

    assert(
      voterRegistrationRequests.some(
        (r) => r.id === voterRegistrationRequestId
      ),
      `no voter registration request found with ID ${voterRegistrationRequestId}`
    );

    this.client.run(
      `
      insert into voter_election_registrations (
        id,
        voter_id,
        voter_registration_request_id,
        election_id,
        precinct_id,
        ballot_style_id
      ) values (
        ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      voterId,
      voterRegistrationRequestId,
      electionId,
      precinctId,
      ballotStyleId
    );

    return id;
  }

  /**
   * Records votes for a voter registration.
   */
  createVoterSelectionForVoterElectionRegistration({
    voterId,
    voterElectionRegistrationId,
    votes,
  }: {
    voterId: Id;
    voterElectionRegistrationId: Id;
    votes: VotesDict;
  }): void {
    const id = uuid();

    // TODO: validate votes against election definition
    this.client.run(
      `
      insert into voter_election_selections (
        id,
        voter_id,
        voter_election_registration_id,
        votes_json
      ) values (
        ?, ?, ?, ?
      )
      `,
      id,
      voterId,
      voterElectionRegistrationId,
      JSON.stringify(votes)
    );
  }

  /**
   * Gets the voter selection for the given voter registration ID.
   */
  getVoterSelectionForVoterElectionRegistration(
    voterElectionRegistrationId: Id
  ): Optional<VotesDict> {
    const result = this.client.one(
      `
      select
        votes_json as votesJson
      from voter_election_selections
      where voter_election_registration_id = ?
      order by created_at desc
      `,
      voterElectionRegistrationId
    ) as Optional<{ votesJson: string }>;

    if (!result) {
      return undefined;
    }

    return JSON.parse(result.votesJson);
  }
}
