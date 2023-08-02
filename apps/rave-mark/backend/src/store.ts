import { VX_MACHINE_ID } from '@votingworks/backend';
import { assert, Optional } from '@votingworks/basics';
import { Client as DbClient } from '@votingworks/db';
import {
  BallotStyleId,
  CVR,
  ElectionDefinition,
  Id,
  PrecinctId,
  safeParseElectionDefinition,
  SystemSettings,
  SystemSettingsDbRow,
  VotesDict,
} from '@votingworks/types';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { join } from 'path';
import {
  PrintedBallot,
  PrintedBallotRow,
  ClientId,
  deserializePrintedBallot,
  deserializeElection,
  deserializeRegistration,
  deserializeRegistrationRequest,
  deserializeServerSyncAttempt,
  Election,
  ElectionRow,
  Registration,
  RegistrationRequest,
  RegistrationRequestRow,
  RegistrationRow,
  ServerId,
  ServerSyncAttempt,
  ServerSyncAttemptRow,
  ScannedBallot,
  ScannedBallotRow,
  deserializeScannedBallot,
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
   * Gets an election.
   */
  getElection({ clientId }: { clientId: ClientId }): Optional<Election>;
  getElection({ serverId }: { serverId: ServerId }): Optional<Election>;
  getElection({
    clientId,
    serverId,
  }: {
    clientId?: ClientId;
    serverId?: ServerId;
  }): Optional<Election> {
    const id = clientId ?? serverId;
    assert(id, 'Must provide either clientId or serverId');
    const electionRow = this.client.one(
      `select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        election
      from elections
      where ${clientId ? 'client_id' : 'server_id'} = ?
      `,
      id
    ) as Optional<ElectionRow>;

    return electionRow ? deserializeElection(electionRow) : undefined;
  }

  /**
   * Creates a new election record.
   */
  createElection({
    id,
    serverId,
    clientId,
    machineId = VX_MACHINE_ID,
    election,
  }: {
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    election: string;
  }): ClientId {
    assert(
      (serverId === undefined) === (clientId === undefined),
      'election serverId must be defined if and only if clientId is defined'
    );
    assert(
      (machineId === VX_MACHINE_ID) === (clientId === id || !clientId),
      'election machineId must be VX_MACHINE_ID if and only if ID equals clientId'
    );

    safeParseElectionDefinition(election).assertOk(
      `Unable to parse election data: ${election}`
    );

    this.client.run(
      `
      insert or replace into elections (
        id,
        server_id,
        client_id,
        machine_id,
        election
      ) values (
        ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      election
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
  isAdmin(commonAccessCardId: Id): boolean {
    const result = this.client.one(
      `
      select
        count(*) as count
      from admins
      where common_access_card_id = ?
      `,
      commonAccessCardId
    ) as { count: number };

    return result.count > 0;
  }

  /**
   * Makes a user with the given CAC ID an admin.
   */
  createAdmin({
    commonAccessCardId,
    createdAt,
  }: {
    commonAccessCardId: Id;
    createdAt?: DateTime;
  }): void {
    this.client.run(
      `
      insert or replace into admins (
        common_access_card_id,
        created_at
      ) values (
        ?, ?
      )
      `,
      commonAccessCardId,
      (createdAt ?? DateTime.utc()).toSQL()
    );
  }

  /**
   * Clears all admin users.
   */
  resetAdmins(): void {
    this.client.run('delete from admins');
  }

  /**
   * Gets all the registrations for a given voter by CAC ID.
   */
  getRegistrationRequests(commonAccessCardId: Id): RegistrationRequest[] {
    const result = this.client.all(
      `
      select
        id as id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        given_name as givenName,
        family_name as familyName,
        address_line_1 as addressLine1,
        address_line_2 as addressLine2,
        city as city,
        state as state,
        postal_code as postalCode,
        state_id as stateId,
        created_at as createdAt
      from registration_requests
      where common_access_card_id = ?
      `,
      commonAccessCardId
    ) as RegistrationRequestRow[];

    return result.map(deserializeRegistrationRequest);
  }

  /**
   * @returns registrations sorted by most recent first
   */
  getRegistrations(commonAccessCardId: Id): Registration[] {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        registration_request_id as registrationRequestId,
        election_id as electionId,
        precinct_id as precinctId,
        ballot_style_id as ballotStyleId,
        created_at as createdAt
      from registrations
      where common_access_card_id = ?
      order by created_at desc
      `,
      commonAccessCardId
    ) as RegistrationRow[];

    return result.map(deserializeRegistration);
  }

  /**
   * Gets the election for the given voter registration ID.
   */
  getRegistrationElection(
    registrationId: ClientId
  ): Optional<ElectionDefinition> {
    const result = this.client.one(
      `
      select
        election
      from registrations
      inner join elections on elections.id = registrations.election_id
      where registrations.id = ?
      `,
      registrationId
    ) as Optional<{ election: string | Buffer }>;

    if (!result) {
      return undefined;
    }

    const electionDefinitionParseResult = safeParseElectionDefinition(
      result.election.toString()
    );

    if (electionDefinitionParseResult.isErr()) {
      throw new Error('Unable to parse stored election data.');
    }

    return electionDefinitionParseResult.ok();
  }

  /**
   * Creates a registration request for the voter with the given CAC ID.
   */
  createRegistrationRequest({
    id,
    serverId,
    clientId,
    machineId = VX_MACHINE_ID,
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
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    commonAccessCardId: Id;
    givenName: string;
    familyName: string;
    addressLine1: string;
    addressLine2?: string;
    city: string;
    state: string;
    postalCode: string;
    stateId: string;
  }): ClientId {
    assert(
      (serverId === undefined) === (clientId === undefined),
      'registration request serverId must be defined if and only if clientId is defined'
    );
    assert(
      (machineId === VX_MACHINE_ID) === (clientId === id || !clientId),
      'registration request machineId must be VX_MACHINE_ID if and only if ID equals clientId'
    );

    this.client.run(
      `
      insert or replace into registration_requests (
        id,
        server_id,
        client_id,
        machine_id,
        common_access_card_id,
        given_name,
        family_name,
        address_line_1,
        address_line_2,
        city,
        state,
        postal_code,
        state_id
      ) values (
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      commonAccessCardId,
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
   * Associates a registration with an election.
   */
  createRegistration({
    id,
    serverId,
    clientId,
    machineId = VX_MACHINE_ID,
    registrationRequestId,
    electionId,
    precinctId,
    ballotStyleId,
  }: {
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    registrationRequestId: ClientId;
    electionId: ClientId;
    precinctId: PrecinctId;
    ballotStyleId: BallotStyleId;
  }): ClientId {
    assert(
      (serverId === undefined) === (clientId === undefined),
      'registration serverId must be defined if and only if clientId is defined'
    );
    assert(
      (machineId === VX_MACHINE_ID) === (clientId === id || !clientId),
      'registration machineId must be VX_MACHINE_ID if and only if ID equals clientId'
    );

    const registrationRequest = this.getRegistrationRequest({
      clientId: registrationRequestId,
    });
    assert(
      registrationRequest,
      `registration request ${registrationRequestId} not found`
    );

    this.client.run(
      `
      insert or replace into registrations (
        id,
        server_id,
        client_id,
        machine_id,
        common_access_card_id,
        registration_request_id,
        election_id,
        precinct_id,
        ballot_style_id
      ) values (
        ?, ?, ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      registrationRequest.commonAccessCardId,
      registrationRequest.id,
      electionId,
      precinctId,
      ballotStyleId
    );

    return id;
  }

  getRegistrationRequest({
    serverId,
  }: {
    serverId: ServerId;
  }): Optional<RegistrationRequest>;
  getRegistrationRequest({
    clientId,
  }: {
    clientId: ClientId;
  }): Optional<RegistrationRequest>;
  getRegistrationRequest({
    serverId,
    clientId,
  }: {
    serverId?: ServerId;
    clientId?: ClientId;
  }): Optional<RegistrationRequest> {
    const id = serverId ?? clientId;
    assert(id !== undefined, 'serverId or clientId must be defined');

    const result = this.client.one(
      `
      select
        id as id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        given_name as givenName,
        family_name as familyName,
        address_line_1 as addressLine1,
        address_line_2 as addressLine2,
        city as city,
        state as state,
        postal_code as postalCode,
        state_id as stateId,
        created_at as createdAt
      from registration_requests
      where ${serverId ? 'server_id' : 'client_id'} = ?
      `,
      id
    ) as Optional<RegistrationRequestRow>;

    return result ? deserializeRegistrationRequest(result) : undefined;
  }

  /**
   * Records a cast ballot for a voter registration.
   */
  createPrintedBallot({
    id,
    serverId,
    clientId,
    machineId = VX_MACHINE_ID,
    registrationId,
    castVoteRecord,
  }: {
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    registrationId: ClientId;
    castVoteRecord: CVR.CVR;
  }): ClientId {
    assert(
      (serverId === undefined) === (clientId === undefined),
      'ballot serverId must be defined if and only if clientId is defined'
    );
    assert(
      (machineId === VX_MACHINE_ID) === (clientId === id || !clientId),
      'ballot machineId must be VX_MACHINE_ID if and only if ID equals clientId'
    );

    const registration = this.getRegistration({ clientId: registrationId });
    assert(
      registration,
      `no registration found with client ID ${registrationId}`
    );

    // TODO: validate votes against election definition
    this.client.run(
      `
      insert or replace into ballots (
        id,
        server_id,
        client_id,
        machine_id,
        common_access_card_id,
        registration_id,
        cast_vote_record
      ) values (
        ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      registration.commonAccessCardId,
      registrationId,
      JSON.stringify(castVoteRecord)
    );

    return id;
  }

  getRegistration({
    clientId,
  }: {
    clientId?: ClientId;
  }): Optional<Registration>;
  getRegistration({
    serverId,
  }: {
    serverId?: ServerId;
  }): Optional<Registration>;
  getRegistration({
    clientId,
    serverId,
  }: {
    clientId?: ClientId;
    serverId?: ServerId;
  }): Optional<Registration> {
    const id = clientId ?? serverId;
    assert(id !== undefined, 'clientId or serverId must be defined');

    const result = this.client.one(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        registration_request_id as registrationRequestId,
        election_id as electionId,
        precinct_id as precinctId,
        ballot_style_id as ballotStyleId,
        created_at as createdAt
      from registrations
      where ${clientId ? 'client_id' : 'server_id'} = ?
      `,
      id
    ) as Optional<RegistrationRow>;

    return result ? deserializeRegistration(result) : undefined;
  }

  /**
   * Gets the voter selection for the given voter registration ID.
   */
  getPrintedBallotCastVoteRecordForRegistration(
    registrationId: ClientId
  ): Optional<VotesDict> {
    const result = this.client.one(
      `
      select
        cast_vote_record as castVoteRecordJson
      from printed_ballots
      where registration_id = ?
      order by created_at desc
      `,
      registrationId
    ) as Optional<{ castVoteRecordJson: string }>;

    if (!result) {
      return undefined;
    }

    return JSON.parse(result.castVoteRecordJson);
  }

  /**
   * Creates a new server synchronization attempt record.
   */
  createServerSyncAttempt({
    creator,
    trigger,
    initialStatusMessage,
  }: {
    creator: string;
    trigger: 'manual' | 'scheduled';
    initialStatusMessage: string;
  }): ClientId {
    const id = ClientId();
    this.client.run(
      `
      insert into server_sync_attempts (
        id,
        creator,
        trigger,
        status_message
      ) values (
        ?, ?, ?, ?
      )
      `,
      id,
      creator,
      trigger,
      initialStatusMessage
    );
    return id;
  }

  /**
   * Updates the state for the given server synchronization attempt.
   */
  updateServerSyncAttempt({
    id,
    status,
    statusMessage,
  }: {
    id: ClientId;
    status: 'pending' | 'success' | 'failure';
    statusMessage: string;
  }): void {
    this.client.run(
      `
      update server_sync_attempts
      set
        status_message = ?,
        success = ?,
        completed_at = ?
      where id = ?
      `,
      statusMessage,
      status === 'pending' ? null : status === 'success' ? 1 : 0,
      status === 'pending' ? null : DateTime.utc().toSQL(),
      id
    );
  }

  /**
   * Gets the most recent server synchronization attempts.
   */
  getServerSyncAttempts({
    limit = 100,
  }: { limit?: number } = {}): ServerSyncAttempt[] {
    return (
      this.client.all(
        `
        select
          id,
          creator,
          trigger,
          status_message as statusMessage,
          success,
          created_at as createdAt,
          completed_at as completedAt
        from server_sync_attempts
        order by created_at desc
        limit ?
        `,
        limit
      ) as ServerSyncAttemptRow[]
    ).map(deserializeServerSyncAttempt);
  }

  getLastSyncedElectionId(): Optional<ServerId> {
    const result = this.client.one(
      `
      select
        server_id as serverId
      from elections
      where server_id is not null
      order by created_at desc
      `
    ) as Optional<{ serverId: ServerId }>;

    return result ? result.serverId : undefined;
  }

  getLastSyncedRegistrationRequestId(): Optional<ServerId> {
    const result = this.client.one(
      `
      select
        server_id as serverId
      from registration_requests
      where server_id is not null
      order by created_at desc
      `
    ) as Optional<{ serverId: ServerId }>;

    return result ? result.serverId : undefined;
  }

  getLastSyncedRegistrationId(): Optional<ServerId> {
    const result = this.client.one(
      `
      select
        server_id as serverId
      from registrations
      where server_id is not null
      order by created_at desc
      `
    ) as Optional<{ serverId: ServerId }>;

    return result ? result.serverId : undefined;
  }

  getLastSyncedPrintedBallotId(): Optional<ServerId> {
    const result = this.client.one(
      `
      select
        server_id as serverId
      from printed_ballots
      where server_id is not null
      order by created_at desc
      `
    ) as Optional<{ serverId: ServerId }>;

    return result ? result.serverId : undefined;
  }

  getLastSyncedScannedBallotId(): Optional<ServerId> {
    const result = this.client.one(
      `
      select
        server_id as serverId
      from scanned_ballots
      where server_id is not null
      order by created_at desc
      `
    ) as Optional<{ serverId: ServerId }>;

    return result ? result.serverId : undefined;
  }

  getRegistrationRequestsToSync(): RegistrationRequest[] {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        given_name as givenName,
        family_name as familyName,
        address_line_1 as addressLine1,
        address_line_2 as addressLine2,
        city,
        state,
        postal_code as postalCode,
        state_id as stateId
      from registration_requests
      where server_id is null
      `
    ) as RegistrationRequestRow[];

    return result.map(deserializeRegistrationRequest);
  }

  getRegistrationsToSync(): Registration[] {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        registration_request_id as registrationRequestId,
        election_id as electionId,
        precinct_id as precinctId,
        ballot_style_id as ballotStyleId
      from registrations
      where server_id is null
      `
    ) as RegistrationRow[];

    return result.map(deserializeRegistration);
  }

  getElectionsToSync(): Election[] {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        election
      from elections
      where server_id is null
      `
    ) as ElectionRow[];

    return result.map(deserializeElection);
  }

  getPrintedBallotsToSync(): PrintedBallot[] {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        registration_id as registrationId,
        cast_vote_record as castVoteRecord,
        created_at as createdAt
      from printed_ballots
      where server_id is null
      `
    ) as PrintedBallotRow[];

    return result.map(deserializePrintedBallot);
  }

  getScannedBallotsToSync(): ScannedBallot[] | undefined {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        election_id as electionId,
        cast_vote_record as castVoteRecord,
        created_at as createdAt
      from scanned_ballots
      where server_id is null
      `
    ) as ScannedBallotRow[];

    return result.map(deserializeScannedBallot);
  }
}
