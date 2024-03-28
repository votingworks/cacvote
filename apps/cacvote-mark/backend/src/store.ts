import {
  IteratorPlus,
  Optional,
  Result,
  assert,
  asyncResultBlock,
  iter,
} from '@votingworks/basics';
import { Client as DbClient } from '@votingworks/db';
import {
  SystemSettings,
  safeParse,
  safeParseSystemSettings,
  unsafeParse,
} from '@votingworks/types';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { join } from 'path';
import { ZodError } from 'zod';
import {
  Election,
  JournalEntry,
  JurisdictionCode,
  JurisdictionCodeSchema,
  Registration,
  RegistrationObjectType,
  RegistrationRequest,
  RegistrationRequestObjectType,
  SignedObject,
  Uuid,
  UuidSchema,
} from './cacvote-server/types';

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

  getLatestJournalEntry(): Optional<JournalEntry> {
    const result = this.client.one(
      `
      select id, object_id, jurisdiction, object_type, action, created_at
      from journal_entries
      order by created_at desc
      limit 1`
    ) as Optional<{
      id: string;
      object_id: string;
      jurisdiction: string;
      object_type: string;
      action: string;
      created_at: string;
    }>;

    return result
      ? new JournalEntry(
          safeParse(UuidSchema, result.id).assertOk('assuming valid UUID'),
          safeParse(UuidSchema, result.object_id).assertOk(
            'assuming valid UUID'
          ),
          safeParse(JurisdictionCodeSchema, result.jurisdiction).assertOk(
            'assuming valid jurisdiction code'
          ),
          result.object_type,
          result.action,
          DateTime.fromSQL(result.created_at)
        )
      : undefined;
  }

  getJournalEntries(): JournalEntry[] {
    const rows = this.client.all(
      `select id, object_id, jurisdiction, object_type, action, created_at
      from journal_entries
      order by created_at`
    ) as Array<{
      id: string;
      object_id: string;
      jurisdiction: string;
      object_type: string;
      action: string;
      created_at: string;
    }>;

    return rows.map(
      (row) =>
        new JournalEntry(
          unsafeParse(UuidSchema, row.id),
          unsafeParse(UuidSchema, row.object_id),
          unsafeParse(JurisdictionCodeSchema, row.jurisdiction),
          row.object_type,
          row.action,
          DateTime.fromSQL(row.created_at)
        )
    );
  }

  /**
   * Adds journal entries to the store.
   */
  addJournalEntries(entries: JournalEntry[]): void {
    this.client.transaction(() => {
      const stmt = this.client.prepare(
        `insert into journal_entries (id, object_id, jurisdiction, object_type, action, created_at)
        values (?, ?, ?, ?, ?, ?)`
      );

      for (const entry of entries) {
        stmt.run(
          entry.getId(),
          entry.getObjectId(),
          entry.getJurisdictionCode(),
          entry.getObjectType(),
          entry.getAction(),
          entry.getCreatedAt().toSQL()
        );
      }
    });
  }

  /**
   * Adds an object to the store.
   */
  async addObject(
    object: SignedObject
  ): Promise<Result<Uuid, SyntaxError | ZodError>> {
    return asyncResultBlock(async (bail) => {
      const jurisdiction = (await object.getJurisdictionCode()).okOrElse(bail);
      const payload = object.getPayload().okOrElse(bail);

      this.client.run(
        `insert into objects (id, jurisdiction, object_type, payload, certificates, signature)
        values (?, ?, ?, ?, ?, ?)`,
        object.getId(),
        jurisdiction,
        payload.getObjectType(),
        object.getPayloadRaw(),
        object.getCertificates(),
        object.getSignature()
      );

      return object.getId();
    });
  }

  /**
   * Adds an object to the store from the server.
   */
  async addObjectFromServer(
    object: SignedObject
  ): Promise<Result<Uuid, SyntaxError | ZodError>> {
    return asyncResultBlock(async (bail) => {
      const jurisdiction = (await object.getJurisdictionCode()).okOrElse(bail);
      const payload = object.getPayload().okOrElse(bail);

      this.client.run(
        `insert into objects (id, jurisdiction, object_type, payload, certificates, signature, server_synced_at)
        values (?, ?, ?, ?, ?, ?, current_timestamp)`,
        object.getId(),
        jurisdiction,
        payload.getObjectType(),
        object.getPayloadRaw(),
        object.getCertificates(),
        object.getSignature()
      );

      return object.getId();
    });
  }

  /**
   * Gets an object from the store by its ID.
   */
  getObjectById(objectId: Uuid): Optional<SignedObject> {
    const row = this.client.one(
      `select id, payload, certificates, signature from objects where id = ?`,
      objectId
    ) as Optional<{
      id: string;
      payload: Buffer;
      certificates: Buffer;
      signature: Buffer;
    }>;

    return row
      ? new SignedObject(
          unsafeParse(UuidSchema, row.id),
          row.payload,
          row.certificates,
          row.signature
        )
      : undefined;
  }

  getJournalEntriesForObjectsToPull(): JournalEntry[] {
    const objectTypesToPull = [
      RegistrationRequestObjectType,
      'Registration',
      'Election',
    ];
    const action = 'create';

    const rows = this.client.all(
      `select je.id, je.object_id, je.jurisdiction, je.object_type, je.created_at
      from journal_entries je
      left join objects o on je.object_id = o.id
      where je.object_type in (${objectTypesToPull.map(() => '?').join(', ')})
      and je.action = ?
      and o.id is null
      order by je.created_at`,
      ...objectTypesToPull,
      action
    ) as Array<{
      id: string;
      object_id: string;
      jurisdiction: string;
      object_type: string;
      created_at: string;
    }>;

    return rows.map(
      (row) =>
        new JournalEntry(
          unsafeParse(UuidSchema, row.id),
          unsafeParse(UuidSchema, row.object_id),
          unsafeParse(JurisdictionCodeSchema, row.jurisdiction),
          row.object_type,
          action,
          DateTime.fromSQL(row.created_at)
        )
    );
  }

  /**
   * Gets all unsynced objects from the store.
   */
  getObjectsToPush(): SignedObject[] {
    const rows = this.client.all(
      `select id, payload, certificates, signature from objects where server_synced_at is null`
    ) as Array<{
      id: string;
      payload: Buffer;
      certificates: Buffer;
      signature: Buffer;
    }>;

    return rows.map(
      (row) =>
        new SignedObject(
          unsafeParse(UuidSchema, row.id),
          row.payload,
          row.certificates,
          row.signature
        )
    );
  }

  /**
   * Marks an object as synced with the server.
   */
  markObjectAsSynced(id: Uuid): void {
    this.client.run(
      `update objects set server_synced_at = current_timestamp where id = ?`,
      id
    );
  }

  forEachElection(): IteratorPlus<{
    object: SignedObject;
    election: Election;
  }> {
    return this.forEachObjectOfType('Election').filterMap((object) => {
      const election = object.getPayload().unsafeUnwrap().getData();
      assert(
        election instanceof Election,
        'payload matches object type because we used forEachObjectOfType'
      );
      return { object, election };
    });
  }

  forEachRegistrationRequest({
    commonAccessCardId,
  }: {
    commonAccessCardId: string;
  }): IteratorPlus<{
    object: SignedObject;
    registrationRequest: RegistrationRequest;
  }> {
    return this.forEachObjectOfType(RegistrationRequestObjectType).filterMap(
      (object) => {
        const registrationRequest = object
          .getPayload()
          .unsafeUnwrap()
          .getData();
        assert(
          registrationRequest instanceof RegistrationRequest,
          'payload matches object type because we used forEachObjectType'
        );
        if (
          registrationRequest.getCommonAccessCardId() === commonAccessCardId
        ) {
          return { object, registrationRequest };
        }
      }
    );
  }

  forEachRegistration({
    commonAccessCardId,
    registrationRequestObjectId,
  }: {
    commonAccessCardId: string;
    registrationRequestObjectId?: Uuid;
  }): IteratorPlus<{
    object: SignedObject;
    registration: Registration;
  }> {
    return this.forEachObjectOfType(RegistrationObjectType).filterMap(
      (object) => {
        const registration = object.getPayload().unsafeUnwrap().getData();
        assert(
          registration instanceof Registration,
          'payload matches object type because we used forEachObjectType'
        );
        if (
          registration.getCommonAccessCardId() === commonAccessCardId &&
          (!registrationRequestObjectId ||
            registrationRequestObjectId ===
              registration.getRegistrationRequestObjectId())
        ) {
          return { object, registration };
        }
      }
    );
  }

  forEachObjectOfType(objectType: string): IteratorPlus<SignedObject> {
    // FIXME: this should be using `this.client.each`, but there seems to be a race condition
    // that results in errors with "This database connection is busy executing a query"
    const rows = this.client.all(
      `select id, payload, certificates, signature from objects
        where json_extract(payload, '$.objectType') = ?`,
      objectType
    ) as Array<{
      id: string;
      payload: Buffer;
      certificates: Buffer;
      signature: Buffer;
    }>;
    return iter(rows).map(
      (row) =>
        new SignedObject(
          unsafeParse(UuidSchema, row.id),
          row.payload,
          row.certificates,
          row.signature
        )
    );
  }

  getJurisdictionCodes(): JurisdictionCode[] {
    const rows = this.client.all(
      `select distinct jurisdiction from objects`
    ) as Array<{ jurisdiction: string }>;

    return rows.map((row) =>
      unsafeParse(JurisdictionCodeSchema, row.jurisdiction)
    );
  }
}
