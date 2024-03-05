import { VX_MACHINE_ID } from '@votingworks/backend';
import { assert, Optional } from '@votingworks/basics';
import { Client as DbClient } from '@votingworks/db';
import {
  BallotStyleId,
  ElectionDefinition,
  Id,
  PrecinctId,
  safeParseElectionDefinition,
  safeParseSystemSettings,
  SystemSettings,
  unsafeParse,
  VotesDict,
} from '@votingworks/types';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { join } from 'path';
import {
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
  Jurisdiction,
  JurisdictionRow,
  deserializeJurisdiction,
} from './types/db';
import {
  Base64StringSchema,
  ElectionInput,
  JurisdictionInput,
  PrintedBallotInput,
} from './types/sync';

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
        definition
      from elections
      where ${clientId ? 'client_id' : 'server_id'} = ?
      `,
      id
    ) as Optional<ElectionRow>;

    return electionRow ? deserializeElection(electionRow) : undefined;
  }

  /**
   * Creates a new jurisdiction record or updates an existing one.
   */
  createJurisdiction({
    id,
    name,
    createdAt,
  }: {
    id: ServerId;
    name: string;
    createdAt: DateTime;
  }): void {
    this.client.run(
      `
      insert or replace into jurisdictions (
        id,
        name,
        created_at
      ) values (
        ?, ?, ?
      )
      `,
      id,
      name,
      createdAt.toSQL()
    );
  }

  /**
   * Creates a new election record.
   */
  createElection({
    id,
    serverId,
    clientId,
    machineId = VX_MACHINE_ID,
    jurisdictionId,
    definition,
  }: {
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    jurisdictionId: ServerId;
    definition: Buffer;
  }): ClientId {
    assert(
      (serverId === undefined) === (clientId === undefined),
      'election serverId must be defined if and only if clientId is defined'
    );
    assert(
      (machineId === VX_MACHINE_ID) === (clientId === id || !clientId),
      'election machineId must be VX_MACHINE_ID if and only if ID equals clientId'
    );

    const electionData = definition.toString('utf-8');
    safeParseElectionDefinition(electionData).assertOk(
      `Unable to parse election data: ${electionData}`
    );

    this.client.run(
      `
      insert or replace into elections (
        id,
        server_id,
        client_id,
        machine_id,
        jurisdiction_id,
        definition
      ) values (
        ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      jurisdictionId,
      definition
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
      `insert into system_settings (data) values (?)`,
      JSON.stringify(systemSettings)
    );
  }

  /**
   * Gets system settings or undefined if they aren't loaded yet
   */
  getSystemSettings(): SystemSettings | undefined {
    const result = this.client.one(
      `select data from system_settings`
    ) as Optional<{ data: string }>;

    if (!result) {
      return undefined;
    }
    return safeParseSystemSettings(result.data).unsafeUnwrap();
  }

  /**
   * Delete recently cast ballots (just for testing).
   */
  deleteRecentlyCastBallots(ageInSeconds: number): void {
    assert(ageInSeconds > 0, 'ageInSeconds must be positive');
    this.client.run(
      `delete from printed_ballots where unixepoch(current_timestamp) - unixepoch(created_at) > ${ageInSeconds}`
    );
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
        definition
      from registrations
      inner join elections on elections.id = registrations.election_id
      where registrations.id = ?
      `,
      registrationId
    ) as Optional<{ definition: string | Buffer }>;

    if (!result) {
      return undefined;
    }

    const electionDefinitionParseResult = safeParseElectionDefinition(
      result.definition.toString()
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
    jurisdictionId,
    commonAccessCardId,
    givenName,
    familyName,
  }: {
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    jurisdictionId: ServerId;
    commonAccessCardId: Id;
    givenName: string;
    familyName: string;
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
        jurisdiction_id,
        common_access_card_id,
        given_name,
        family_name
      ) values (
        ?, ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      jurisdictionId,
      commonAccessCardId,
      givenName,
      familyName
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
    jurisdictionId,
    electionId,
    precinctId,
    ballotStyleId,
  }: {
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    registrationRequestId: ClientId;
    jurisdictionId: ServerId;
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
        jurisdiction_id,
        election_id,
        precinct_id,
        ballot_style_id
      ) values (
        ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      registrationRequest.commonAccessCardId,
      registrationRequest.id,
      jurisdictionId,
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
        jurisdiction_id as jurisdictionId,
        common_access_card_id as commonAccessCardId,
        given_name as givenName,
        family_name as familyName,
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
    commonAccessCardCertificate,
    castVoteRecord,
    castVoteRecordSignature,
  }: {
    id: ClientId;
    serverId?: ServerId;
    clientId?: ClientId;
    machineId?: Id;
    registrationId: ClientId;
    commonAccessCardCertificate: Buffer;
    castVoteRecord: Buffer;
    castVoteRecordSignature: Buffer;
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
      insert or replace into printed_ballots (
        id,
        server_id,
        client_id,
        machine_id,
        common_access_card_id,
        common_access_card_certificate,
        registration_id,
        cast_vote_record,
        cast_vote_record_signature
      ) values (
        ?, ?, ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      serverId ?? null,
      clientId ?? id,
      machineId,
      registration.commonAccessCardId,
      commonAccessCardCertificate,
      registration.id,
      castVoteRecord,
      castVoteRecordSignature
    );

    return id;
  }

  /**
   * Records a cast ballot for a voter registration.
   */
  createCastBallot({
    id,
    registrationId,
    commonAccessCardCertificate,
    castVoteRecord,
    castVoteRecordSignature,
  }: {
    id: ClientId;
    registrationId: ClientId;
    commonAccessCardCertificate: Buffer;
    castVoteRecord: Buffer;
    castVoteRecordSignature: Buffer;
  }): ClientId {
    const registration = this.getRegistration({ clientId: registrationId });
    assert(
      registration,
      `no registration found with client ID ${registrationId}`
    );

    // TODO: validate votes against election definition
    this.client.run(
      `
      insert or replace into printed_ballots (
        id,
        client_id,
        machine_id,
        common_access_card_id,
        common_access_card_certificate,
        registration_id,
        cast_vote_record,
        cast_vote_record_signature
      ) values (
        ?, ?, ?, ?, ?, ?, ?, ?
      )
      `,
      id,
      id,
      VX_MACHINE_ID,
      registration.commonAccessCardId,
      commonAccessCardCertificate,
      registration.id,
      castVoteRecord,
      castVoteRecordSignature
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
   * Gets the sync status, i.e. the count of the synced and pending items of
   * various types.
   */
  getSyncStatus(): {
    printedBallots: { synced: number; pending: number };
    pendingRegistrations: { synced: number; pending: number };
    elections: { synced: number; pending: number };
  } {
    const printedBallots = this.client.one(
      `
      select
        sum(case when server_id is not null then 1 else 0 end) as synced,
        sum(case when server_id is null then 1 else 0 end) as pending
      from printed_ballots
      `
    ) as { synced: number; pending: number };

    const pendingRegistrations = this.client.one(
      `
      select
        sum(case when server_id is not null then 1 else 0 end) as synced,
        sum(case when server_id is null then 1 else 0 end) as pending
      from registrations
      `
    ) as { synced: number; pending: number };

    const elections = this.client.one(
      `
      select
        sum(case when server_id is not null then 1 else 0 end) as synced,
        sum(case when server_id is null then 1 else 0 end) as pending
      from elections
      `
    ) as { synced: number; pending: number };

    return {
      printedBallots,
      pendingRegistrations,
      elections,
    };
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

  getRegistrationRequestsToSync(): RegistrationRequest[] {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        jurisdiction_id as jurisdictionId,
        common_access_card_id as commonAccessCardId,
        given_name as givenName,
        family_name as familyName
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
        jurisdiction_id as jurisdictionId,
        common_access_card_id as commonAccessCardId,
        (select client_id from registration_requests where id = registration_request_id) as registrationRequestId,
        (select client_id from elections where id = election_id) as electionId,
        precinct_id as precinctId,
        ballot_style_id as ballotStyleId
      from registrations
      where server_id is null
      `
    ) as RegistrationRow[];

    return result.map(deserializeRegistration);
  }

  getJurisdictions(): Jurisdiction[] {
    const result = this.client.all(
      `
      select
        id,
        name,
        created_at as createdAt
      from jurisdictions
      `
    ) as JurisdictionRow[];

    return result.map(deserializeJurisdiction);
  }

  getJurisdictionsToSync(): JurisdictionInput[] {
    // we don't create jurisdictions in the client, so there's nothing to sync
    return [];
  }

  getElectionsToSync(): ElectionInput[] {
    const result = this.client.all(
      `
      select
        id,
        jurisdiction_id as jurisdictionId,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        definition
      from elections
      where server_id is null
      `
    ) as ElectionRow[];

    return result.map((row) => {
      const record = deserializeElection(row);
      return {
        ...record,
        definition: unsafeParse(
          Base64StringSchema,
          record.definition.toString('base64')
        ),
      };
    });
  }

  getPrintedBallotsToSync(): PrintedBallotInput[] {
    const result = this.client.all(
      `
      select
        id,
        server_id as serverId,
        client_id as clientId,
        machine_id as machineId,
        common_access_card_id as commonAccessCardId,
        common_access_card_certificate as commonAccessCardCertificate,
        (select client_id from registrations where id = registration_id) as registrationId,
        cast_vote_record as castVoteRecord,
        cast_vote_record_signature as castVoteRecordSignature,
        created_at as createdAt
      from printed_ballots
      where server_id is null
      `
    ) as PrintedBallotRow[];

    return result.map((row) => {
      const record = deserializePrintedBallot(row);
      return {
        ...record,
        commonAccessCardCertificate: unsafeParse(
          Base64StringSchema,
          record.commonAccessCardCertificate.toString('base64')
        ),
        castVoteRecord: unsafeParse(
          Base64StringSchema,
          record.castVoteRecord.toString('base64')
        ),
        castVoteRecordSignature: unsafeParse(
          Base64StringSchema,
          record.castVoteRecordSignature.toString('base64')
        ),
      };
    });
  }
}
