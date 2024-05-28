import {
  Optional,
  Result,
  asyncResultBlock,
  err,
  ok,
} from '@votingworks/basics';
import fetch, { Headers, Request } from 'cross-fetch';
import { safeParse, safeParseJson } from '@votingworks/types';
import { ZodError, z } from 'zod';
import {
  JournalEntry,
  JournalEntrySchema,
  SignedObject,
  SignedObjectSchema,
  Uuid,
  UuidSchema,
} from './types';

export type ClientError =
  | { type: 'network'; message: string }
  | { type: 'schema'; error: SyntaxError | ZodError; message: string };

export type ClientResult<T> = Result<T, ClientError>;

export class Client {
  constructor(private readonly baseUrl: URL) {}

  /**
   * Create a new client to connect to the server running on localhost.
   */
  static localhost(): Client {
    return new Client(new URL('http://localhost:8000'));
  }

  /**
   * Check that the server is responding.
   */
  async checkStatus(): Promise<ClientResult<void>> {
    return asyncResultBlock(async (bail) => {
      const response = (await this.get('/api/status')).okOrElse(bail);

      if (!response.ok) {
        bail({ type: 'network', message: response.statusText });
      }
    });
  }

  /**
   * Create an object on the server.
   */
  async createObject(signedObject: SignedObject): Promise<ClientResult<Uuid>> {
    return asyncResultBlock(async (bail) => {
      const response = (
        await this.post('/api/objects', JSON.stringify(signedObject, null, 2))
      ).okOrElse(bail);

      if (!response.ok) {
        bail({ type: 'network', message: response.statusText });
      }

      return safeParse(UuidSchema, await response.text()).okOrElse<ZodError>(
        (error) => bail({ type: 'schema', error, message: error.message })
      );
    });
  }

  /**
   * Retrieve an object from the server.
   */
  async getObjectById(
    uuid: Uuid
  ): Promise<ClientResult<Optional<SignedObject>>> {
    return asyncResultBlock(async (bail) => {
      const response = (await this.get(`/api/objects/${uuid}`)).okOrElse(bail);

      if (response.status === 404) {
        // not found
        return undefined;
      }

      if (!response.ok) {
        bail({ type: 'network', message: response.statusText });
      }

      const json = await response.text();
      return safeParseJson(json, SignedObjectSchema).okOrElse<ZodError>(
        (error) =>
          bail({
            type: 'schema',
            error,
            message: error.message,
          })
      );
    });
  }

  /**
   * Get journal entries from the server.
   *
   * @example
   *
   * ```ts
   * const client = Client.localhost();
   *
   * // Get all journal entries.
   * const journalEntries = await client.getJournalEntries();
   *
   * // Get journal entries since a specific entry.
   * const journalEntriesSince = await client.getJournalEntries(journalEntries[0].getId());
   * ```
   */
  async getJournalEntries(since?: Uuid): Promise<ClientResult<JournalEntry[]>> {
    return asyncResultBlock(async (bail) => {
      const response = (
        await this.get(`/api/journal-entries${since ? `?since=${since}` : ''}`)
      ).okOrElse(bail);

      if (!response.ok) {
        bail({ type: 'network', message: response.statusText });
      }

      return safeParse(
        z.array(JournalEntrySchema),
        await response.json()
      ).okOrElse<ZodError>((error) =>
        bail({
          type: 'schema',
          error,
          message: error.message,
        })
      );
    });
  }

  private async get(path: string): Promise<ClientResult<Response>> {
    try {
      return ok(await fetch(new URL(path, this.baseUrl)));
    } catch (error) {
      return err({ type: 'network', message: (error as Error).message });
    }
  }

  private async post(
    path: string,
    body: BodyInit
  ): Promise<ClientResult<Response>> {
    const request = new Request(new URL(path, this.baseUrl), {
      method: 'POST',
      headers: new Headers({
        'Content-Type': 'application/json',
      }),
      body,
    });

    try {
      return ok(await fetch(request));
    } catch (error) {
      return err({ type: 'network', message: (error as Error).message });
    }
  }
}
