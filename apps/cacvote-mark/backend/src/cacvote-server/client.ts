import { Buffer } from 'buffer';
import { Result, asyncResultBlock, err, ok } from '@votingworks/basics';
import fetch, { Headers, Request } from 'cross-fetch';
import { safeParse } from '@votingworks/types';
import { ZodError, z } from 'zod';
import {
  JournalEntry,
  JournalEntrySchema,
  SignedObject,
  Uuid,
  UuidSchema,
} from './types';

export type ClientError =
  | { type: 'network'; message: string }
  | { type: 'schema'; error: ZodError; message: string };

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
    const statusResult = await this.get('/api/status');

    if (statusResult.isErr()) {
      return statusResult;
    }

    const response = statusResult.ok();
    if (!response.ok) {
      return err({ type: 'network', message: response.statusText });
    }

    return ok();
  }

  /**
   * Create an object on the server.
   */
  async createObject(signedObject: SignedObject): Promise<ClientResult<Uuid>> {
    const postResult = await this.post(
      '/api/objects',
      JSON.stringify(signedObject)
    );

    if (postResult.isErr()) {
      return postResult;
    }

    const response = postResult.ok();
    if (!response.ok) {
      return err({ type: 'network', message: response.statusText });
    }

    const uuidResult = safeParse(UuidSchema, await response.text());

    if (uuidResult.isErr()) {
      return err({
        type: 'schema',
        error: uuidResult.err(),
        message: uuidResult.err().message,
      });
    }

    return uuidResult;
  }

  /**
   * Retrieve an object from the server.
   */
  async getObjectById(uuid: Uuid): Promise<ClientResult<SignedObject>> {
    const getResult = await this.get(`/api/objects/${uuid}`);

    if (getResult.isErr()) {
      return getResult;
    }

    const response = getResult.ok();
    if (!response.ok) {
      return err({ type: 'network', message: response.statusText });
    }

    const signedObject = await response.json();

    return ok(
      new SignedObject(
        Buffer.from(signedObject.payload, 'base64'),
        Buffer.from(signedObject.certificates, 'base64'),
        Buffer.from(signedObject.signature, 'base64')
      )
    );
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
        return err({ type: 'network', message: response.statusText });
      }

      const entries = safeParse(
        z.array(JournalEntrySchema),
        await response.json()
      ).okOrElse<ZodError>((error) =>
        bail({
          type: 'schema',
          error,
          message: error.message,
        })
      );

      return entries.map(
        (entry) =>
          new JournalEntry(
            entry.id,
            entry.objectId,
            entry.jurisdiction,
            entry.objectType,
            entry.action,
            entry.createdAt
          )
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
