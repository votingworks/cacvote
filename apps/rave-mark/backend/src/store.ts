import { Buffer } from 'buffer';
import { v4 as uuid } from 'uuid';
import { assert, Optional } from '@votingworks/basics';
import { Client as DbClient } from '@votingworks/db';
import {
  ElectionDefinition,
  Id,
  safeParseElectionDefinition,
  SystemSettings,
  SystemSettingsDbRow,
  VotesDict,
} from '@votingworks/types';
import { join } from 'path';
import { DateTime } from 'luxon';
import { VoterInfo, VoterRegistrationInfo } from './types/db';

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
   * @param id The database ID of the voter, notably NOT the CAC ID.
   */
  getVoterRegistrations(id: Id): VoterRegistrationInfo[] {
    const result = this.client.all(
      `
      select
        id,
        voter_id as voterId,
        election_id as electionId,
        given_name as givenName,
        family_name as familyName,
        voted_at as votedAt
      from voter_registrations
      where voter_id = ?
      `,
      id
    ) as Array<{
      id: string;
      voterId: string;
      electionId: string | null;
      givenName: string;
      familyName: string;
      votedAt: string | null;
    }>;

    return result.map((row) => ({
      id: row.id,
      voterId: row.voterId,
      electionId: row.electionId ?? undefined,
      givenName: row.givenName,
      familyName: row.familyName,
      votedAt: row.votedAt ? DateTime.fromSQL(row.votedAt) : undefined,
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
      from voter_registrations
      inner join elections on elections.id = voter_registrations.election_id
      where voter_registrations.id = ?
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
   * Creates a voter registration for the voter with the given Common Access
   * Card ID.
   */
  createVoterRegistration({
    commonAccessCardId,
    givenName,
    familyName,
  }: {
    commonAccessCardId: Id;
    givenName: string;
    familyName: string;
  }): Id {
    const voterInfo = this.getOrCreateVoterInfo(commonAccessCardId);
    const id = uuid();

    this.client.run(
      `
      insert into voter_registrations (
        id,
        voter_id,
        given_name,
        family_name
      ) values (
        ?, ?, ?, ?
      )
      `,
      id,
      voterInfo.id,
      givenName,
      familyName
    );

    return id;
  }

  /**
   * Associates a voter registration with an election.
   */
  setVoterRegistrationElection(voterRegistrationId: Id, electionId?: Id): void {
    this.client.run(
      `
      update voter_registrations
      set election_id = ?
      where id = ?
      `,
      electionId ?? null,
      voterRegistrationId
    );
  }

  /**
   * Records votes for a voter registration.
   */
  recordVotesForVoterRegistration({
    voterRegistrationId,
    votes,
  }: {
    voterRegistrationId: Id;
    votes: VotesDict;
  }): void {
    // TODO: validate votes against election definition
    this.client.run(
      `
      update voter_registrations
      set
        voted_at = ?,
        votes_json = ?
      where id = ?
      `,
      DateTime.utc().toSQL(),
      JSON.stringify(votes),
      voterRegistrationId
    );
  }
}
