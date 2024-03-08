import { Optional } from '@votingworks/basics';
import { Client as DbClient } from '@votingworks/db';
import {
  safeParse,
  safeParseSystemSettings,
  SystemSettings,
} from '@votingworks/types';
import { join } from 'path';
import { DateTime } from 'luxon';
import {
  JournalEntry,
  JurisdictionCodeSchema,
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
          entry.getJurisdiction(),
          entry.getObjectType(),
          entry.getAction(),
          entry.getCreatedAt().toSQL()
        );
      }
    });
  }
}
