import { Buffer } from 'buffer';
import {
  Optional,
  Result,
  asyncResultBlock,
  err,
  ok,
} from '@votingworks/basics';
import { safeParseJson } from '@votingworks/types';
import fetch, { Headers, Request } from 'cross-fetch';
import { ZodError, z } from 'zod';
import { cryptography, FileKey, TpmKey } from '@votingworks/auth';
import { DateTime } from 'luxon';
import { Readable } from 'stream';
import { LogEventId, Logger } from '@votingworks/logging';
import {
  CreateSessionRequest,
  CreateSessionRequestPayload,
  CreateSessionResponseSchema,
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

export interface ClientApi {
  /**
   * Check that the server is responding.
   */
  checkStatus(): Promise<ClientResult<void>>;

  /**
   * Create an object on the server.
   */
  createObject(signedObject: SignedObject): Promise<ClientResult<Uuid>>;

  /**
   * Retrieve an object from the server.
   */
  getObjectById(uuid: Uuid): Promise<ClientResult<Optional<SignedObject>>>;

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
  getJournalEntries(since?: Uuid): Promise<ClientResult<JournalEntry[]>>;
}

export class Client implements ClientApi {
  private bearerToken?: string;

  constructor(
    private readonly logger: Logger,
    private readonly baseUrl: URL,
    private readonly signingCert: Buffer,
    private readonly signingPrivateKey: FileKey | TpmKey
  ) {}

  /**
   * Create a new client to connect to the server running on localhost.
   */
  static localhost(
    logger: Logger,
    signingCert: Buffer,
    signingPrivateKey: FileKey | TpmKey
  ): Client {
    return new Client(
      logger,
      new URL('http://localhost:8000'),
      signingCert,
      signingPrivateKey
    );
  }

  /**
   * Check that the server is responding.
   */
  async checkStatus(): Promise<ClientResult<void>> {
    return asyncResultBlock(async (bail) => {
      const response = (await this.get('/api/status')).okOrElse(bail);

      if (!response.ok) {
        void this.logger.log(LogEventId.UnknownError, 'system', {
          message: `checkStatus failed: server responded with status ${response.status}`,
          disposition: 'failure',
        });
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
        await this.withAuthentication(() =>
          this.post('/api/objects', JSON.stringify(signedObject, null, 2))
        )
      ).okOrElse(bail);

      if (!response.ok) {
        void this.logger.log(LogEventId.UnknownError, 'system', {
          message: `createObject failed: server responded with status ${response.status}`,
          disposition: 'failure',
        });
        bail({ type: 'network', message: response.statusText });
      }

      const parsed = UuidSchema.safeParse(await response.text());
      return parsed.success
        ? ok(parsed.data)
        : err({
            type: 'schema',
            error: parsed.error,
            message: parsed.error.message,
          });
    });
  }

  /**
   * Retrieve an object from the server.
   */
  async getObjectById(
    uuid: Uuid
  ): Promise<ClientResult<Optional<SignedObject>>> {
    return asyncResultBlock(async (bail) => {
      const response = (
        await this.withAuthentication(() => this.get(`/api/objects/${uuid}`))
      ).okOrElse(bail);

      if (response.status === 404) {
        // not found
        return undefined;
      }

      if (!response.ok) {
        void this.logger.log(LogEventId.UnknownError, 'system', {
          message: `getObjectById failed: server responded with status ${response.status}`,
          disposition: 'failure',
        });
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
        await this.withAuthentication(
          async () =>
            await this.get(
              `/api/journal-entries${since ? `?since=${since}` : ''}`
            )
        )
      ).okOrElse(bail);

      if (!response.ok) {
        void this.logger.log(LogEventId.UnknownError, 'system', {
          message: `getJournalEntries failed: server responded with status ${response.status}`,
          disposition: 'failure',
        });
        bail({ type: 'network', message: response.statusText });
      }

      const parsed = z
        .array(JournalEntrySchema)
        .safeParse(await response.json());
      return parsed.success
        ? ok(parsed.data)
        : err({
            type: 'schema',
            error: parsed.error,
            message: parsed.error.message,
          });
    });
  }

  private async withAuthentication(
    fn: () => Promise<ClientResult<Response>>
  ): Promise<ClientResult<Response>> {
    return asyncResultBlock(async (bail) => {
      // eslint-disable-next-line no-constant-condition
      while (true) {
        if (!this.bearerToken) {
          void this.logger.log(LogEventId.UnknownError, 'system', {
            message: 'withAuthentication: no bearer token, authenticating',
          });
          (await this.authenticate()).okOrElse(bail);
        }

        const result = await fn();

        if (result.isErr()) {
          return result;
        }

        const response = result.ok();
        if (response.status === 401) {
          this.bearerToken = undefined;
          continue;
        }

        return ok(response);
      }
    });
  }

  private async authenticate(): Promise<ClientResult<void>> {
    return asyncResultBlock(async (bail) => {
      const payload = new CreateSessionRequestPayload(DateTime.now());
      const payloadJson = JSON.stringify(payload);
      const createSessionRequest = new CreateSessionRequest(
        this.signingCert,
        JSON.stringify(payload),
        await cryptography.signMessage({
          message: Readable.from([Buffer.from(payloadJson)]),
          signingPrivateKey: this.signingPrivateKey,
        })
      );
      void this.logger.log(LogEventId.AuthLogin, 'system', {
        message: 'Authenticating with CACvote Server',
      });
      const response = (
        await this.post('/api/sessions', JSON.stringify(createSessionRequest))
      ).okOrElse(bail);

      if (!response.ok) {
        void this.logger.log(LogEventId.AuthLogin, 'system', {
          message: `authenticate failed: server responded with status ${response.status}`,
          disposition: 'failure',
        });
        bail({ type: 'network', message: response.statusText });
      }

      const createSessionResponse = safeParseJson(
        await response.text(),
        CreateSessionResponseSchema
      ).okOrElse<ZodError>((error) =>
        bail({
          type: 'schema',
          error,
          message: error.message,
        })
      );

      void this.logger.log(LogEventId.AuthLogin, 'system', {
        message: `authenticate succeeded: server responded with new bearer token`,
        disposition: 'failure',
      });
      this.bearerToken = createSessionResponse.getBearerToken();
    });
  }

  private async get(path: string): Promise<ClientResult<Response>> {
    try {
      const headers = new Headers();

      const authorizationHeader = this.getAuthorizationHeader();
      if (authorizationHeader) {
        headers.set('Authorization', authorizationHeader);
      }

      const request = new Request(new URL(path, this.baseUrl), {
        method: 'GET',
        headers,
      });

      return ok(await fetch(request));
    } catch (error) {
      return err({ type: 'network', message: (error as Error).message });
    }
  }

  private async post(
    path: string,
    body: BodyInit
  ): Promise<ClientResult<Response>> {
    const headers = new Headers({
      'Content-Type': 'application/json',
    });

    const authorizationHeader = this.getAuthorizationHeader();
    if (authorizationHeader) {
      headers.set('Authorization', authorizationHeader);
    }

    const request = new Request(new URL(path, this.baseUrl), {
      method: 'POST',
      headers,
      body,
    });

    try {
      return ok(await fetch(request));
    } catch (error) {
      return err({ type: 'network', message: (error as Error).message });
    }
  }

  private getAuthorizationHeader(): Optional<string> {
    return this.bearerToken ? `Bearer ${this.bearerToken}` : undefined;
  }
}
