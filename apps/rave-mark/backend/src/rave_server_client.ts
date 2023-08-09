// eslint-disable-next-line max-classes-per-file
import { z } from 'zod';
import { assert } from '@votingworks/basics';
import fetch from 'cross-fetch';
import makeDebug from 'debug';
import {
  BallotStyleId,
  BallotStyleIdSchema,
  CVR,
  Id,
  PrecinctId,
  PrecinctIdSchema,
  unsafeParse,
} from '@votingworks/types';
import { DateTime } from 'luxon';
import { format } from '@votingworks/utils';
import { VX_MACHINE_ID } from '@votingworks/backend';
import { Store } from './store';
import { AuthStatus } from './types/auth';
import { ClientId, ClientIdSchema, ServerId, ServerIdSchema } from './types/db';

const debug = makeDebug('rave-server-client');

export interface RaveServerClient {
  sync(options?: { authStatus: AuthStatus }): Promise<void>;
}

const DateTimeSchema = z
  .string()
  .transform((value) =>
    DateTime.fromISO(value)
  ) as unknown as z.ZodSchema<DateTime>;

interface RegistrationRequestInput {
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: string;
  givenName: string;
  familyName: string;
  addressLine1: string;
  addressLine2?: string;
  city: string;
  state: string;
  postalCode: string;
  stateId: string;
}

type RegistrationRequestOutput = RegistrationRequestInput & {
  serverId: ServerId;
  createdAt: DateTime;
};

const RegistrationRequestOutputSchema: z.ZodSchema<RegistrationRequestOutput> =
  z.object({
    serverId: ServerIdSchema,
    clientId: ClientIdSchema,
    machineId: z.string(),
    commonAccessCardId: z.string(),
    givenName: z.string(),
    familyName: z.string(),
    addressLine1: z.string(),
    addressLine2: z.string().optional(),
    city: z.string(),
    state: z.string(),
    postalCode: z.string(),
    stateId: z.string(),
    createdAt: DateTimeSchema,
  });

interface RegistrationInput {
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: Id;
  registrationRequestId: ClientId;
  electionId: ClientId;
  precinctId: PrecinctId;
  ballotStyleId: BallotStyleId;
}

interface RegistrationOutput {
  serverId: ServerId;
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: Id;
  registrationRequestId: ServerId;
  electionId: ServerId;
  precinctId: PrecinctId;
  ballotStyleId: BallotStyleId;
  createdAt: DateTime;
}

const RegistrationOutputSchema: z.ZodSchema<RegistrationOutput> = z.object({
  serverId: ServerIdSchema,
  clientId: ClientIdSchema,
  machineId: z.string(),
  commonAccessCardId: z.string(),
  registrationRequestId: ServerIdSchema,
  electionId: ServerIdSchema,
  precinctId: PrecinctIdSchema,
  ballotStyleId: BallotStyleIdSchema,
  createdAt: DateTimeSchema,
});

interface PrintedBallotInput {
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: string;
  registrationId: ClientId;
  castVoteRecord: CVR.CVR;
}

interface ScannedBallotInput {
  clientId: ClientId;
  machineId: Id;
  electionId: ClientId;
  castVoteRecord: CVR.CVR;
}

interface PrintedBallotOutput {
  serverId: ServerId;
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: string;
  registrationId: ServerId;
  castVoteRecord: CVR.CVR;
}

const PrintedBallotOutputSchema: z.ZodSchema<PrintedBallotOutput> = z.object({
  serverId: ServerIdSchema,
  clientId: ClientIdSchema,
  machineId: z.string(),
  commonAccessCardId: z.string(),
  registrationId: ServerIdSchema,
  castVoteRecord: CVR.CVRSchema,
});

interface ScannedBallotOutput {
  serverId: ServerId;
  clientId: ClientId;
  machineId: Id;
  electionId: ServerId;
  castVoteRecord: CVR.CVR;
}

const ScannedBallotOutputSchema: z.ZodSchema<ScannedBallotOutput> = z.object({
  serverId: ServerIdSchema,
  clientId: ClientIdSchema,
  machineId: z.string(),
  electionId: ServerIdSchema,
  castVoteRecord: CVR.CVRSchema,
});

interface AdminInput {
  commonAccessCardId: string;
  createdAt: DateTime;
}

type AdminOutput = AdminInput;

const AdminOutputSchema: z.ZodSchema<AdminOutput> = z.object({
  commonAccessCardId: z.string(),
  createdAt: DateTimeSchema,
});

interface ElectionInput {
  clientId: ClientId;
  machineId: Id;
  election: string;
}

type ElectionOutput = ElectionInput & {
  serverId: ServerId;
};

const ElectionOutputSchema: z.ZodSchema<ElectionOutput> = z.object({
  serverId: ServerIdSchema,
  clientId: ClientIdSchema,
  machineId: z.string(),
  election: z.string(),
  createdAt: DateTimeSchema,
});

interface RaveServerSyncInput {
  lastSyncedRegistrationRequestId?: ServerId;
  lastSyncedRegistrationId?: ServerId;
  lastSyncedElectionId?: ServerId;
  lastSyncedPrintedBallotId?: ServerId;
  lastSyncedScannedBallotId?: ServerId;
  registrationRequests?: RegistrationRequestInput[];
  elections?: ElectionInput[];
  registrations?: RegistrationInput[];
  printedBallots?: PrintedBallotInput[];
  scannedBallots?: ScannedBallotInput[];
}

interface RaveServerSyncOutput {
  admins: AdminOutput[];
  elections: ElectionOutput[];
  registrationRequests: RegistrationRequestOutput[];
  registrations: RegistrationOutput[];
  printedBallots: PrintedBallotOutput[];
  scannedBallots: ScannedBallotOutput[];
}

const RaveMarkSyncOutputSchema: z.ZodSchema<RaveServerSyncOutput> = z.object({
  admins: z.array(AdminOutputSchema),
  elections: z.array(ElectionOutputSchema),
  registrationRequests: z.array(RegistrationRequestOutputSchema),
  registrations: z.array(RegistrationOutputSchema),
  printedBallots: z.array(PrintedBallotOutputSchema),
  scannedBallots: z.array(ScannedBallotOutputSchema),
});

function describeSyncInputOrOutput(
  data: RaveServerSyncInput | RaveServerSyncOutput
): string[] {
  const parts: string[] = [];

  if (data.printedBallots?.length) {
    parts.push(
      format.countPhrase(data.printedBallots.length, {
        one: '1 printed ballot',
        many: `${data.printedBallots.length} printed ballots`,
      })
    );
  }

  if (data.scannedBallots?.length) {
    parts.push(
      format.countPhrase(data.scannedBallots.length, {
        one: '1 scanned ballot',
        many: `${data.scannedBallots.length} scanned ballots`,
      })
    );
  }

  if (data.elections?.length) {
    parts.push(
      format.countPhrase(data.elections.length, {
        one: '1 election',
        many: `${data.elections.length} elections`,
      })
    );
  }

  if (data.registrationRequests?.length) {
    parts.push(
      format.countPhrase(data.registrationRequests.length, {
        one: '1 registration request',
        many: `${data.registrationRequests.length} registration requests`,
      })
    );
  }

  if (data.registrations?.length) {
    parts.push(
      format.countPhrase(data.registrations.length, {
        one: '1 registration',
        many: `${data.registrations.length} registrations`,
      })
    );
  }

  return parts;
}

export class RaveServerClientImpl {
  private readonly store: Store;
  private readonly baseUrl: URL;

  constructor({ store, baseUrl }: { store: Store; baseUrl: URL }) {
    this.store = store;
    this.baseUrl = baseUrl;
  }

  async sync({ authStatus }: { authStatus?: AuthStatus } = {}): Promise<void> {
    const user =
      authStatus?.status === 'logged_in' &&
      authStatus.user.role === 'rave_voter'
        ? authStatus.user
        : null;
    assert(!authStatus || user, 'not logged in as voter');
    const creator = user?.commonAccessCardId ?? 'system';

    const syncAttemptId = this.store.createServerSyncAttempt({
      creator,
      trigger: user ? 'manual' : 'scheduled',
      initialStatusMessage: 'Syncing…',
    });

    try {
      const input = this.createSyncInput();
      const output = await this.performSyncRequest(input);
      this.updateLocalStoreFromSyncOutput(output);
      this.updateServerSyncAttempt({ syncAttemptId, input, output });
    } catch (error) {
      debug(
        'RAVE server sync failed: %s',
        error instanceof Error ? error.stack : error
      );
      this.store.updateServerSyncAttempt({
        id: syncAttemptId,
        status: 'failure',
        statusMessage:
          error instanceof Error ? error.message : `unknown error: ${error}`,
      });
    }
  }

  private createServerSyncAttempt(
    creator: string,
    trigger: 'manual' | 'scheduled'
  ): ClientId {
    return this.store.createServerSyncAttempt({
      creator,
      trigger,
      initialStatusMessage: 'Syncing…',
    });
  }

  private createSyncInput(): RaveServerSyncInput {
    const input: RaveServerSyncInput = {
      lastSyncedRegistrationRequestId:
        this.store.getLastSyncedRegistrationRequestId(),
      lastSyncedRegistrationId: this.store.getLastSyncedRegistrationId(),
      lastSyncedElectionId: this.store.getLastSyncedElectionId(),
      lastSyncedPrintedBallotId: this.store.getLastSyncedPrintedBallotId(),
      lastSyncedScannedBallotId: this.store.getLastSyncedScannedBallotId(),
      registrationRequests: this.store.getRegistrationRequestsToSync(),
      elections: this.store.getElectionsToSync(),
      registrations: this.store.getRegistrationsToSync(),
      printedBallots: this.store.getPrintedBallotsToSync(),
      scannedBallots: this.store.getScannedBallotsToSync(),
    };
    debug('RAVE sync input: %O', input);
    return input;
  }

  private async performSyncRequest(
    input: RaveServerSyncInput
  ): Promise<RaveServerSyncOutput> {
    const syncUrl = new URL('api/sync', this.baseUrl);
    const response = await fetch(syncUrl.toString(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(input),
    });
    if (response.status !== 200) {
      const text = await response.text();
      throw new Error(
        `Server responded with ${response.status} ${response.statusText}: ${text}}`
      );
    }

    return this.parseSyncOutput(await response.json());
  }

  private parseSyncOutput(rawOutput: unknown): RaveServerSyncOutput {
    debug('RAVE sync raw output: %O', rawOutput);
    const output = unsafeParse(RaveMarkSyncOutputSchema, rawOutput);
    debug('RAVE sync parsed output: %O', output);
    return output;
  }

  private updateLocalStoreFromSyncOutput(output: RaveServerSyncOutput): void {
    this.store.resetAdmins();
    for (const admin of output.admins) {
      this.store.createAdmin(admin);
    }
    debug('reset and replaced admins; count: %d', output.admins.length);

    for (const election of output.elections) {
      const electionId = this.store.createElection({
        id:
          election.machineId === VX_MACHINE_ID ? election.clientId : ClientId(),
        ...election,
      });
      debug('created or replaced election %s', electionId);
    }

    for (const registrationRequest of output.registrationRequests) {
      const registrationRequestId = this.store.createRegistrationRequest({
        id:
          registrationRequest.machineId === VX_MACHINE_ID
            ? registrationRequest.clientId
            : ClientId(),
        ...registrationRequest,
      });
      debug(
        'created or replaced registration request %s',
        registrationRequestId
      );
    }

    for (const registration of output.registrations) {
      const localRegistrationRequest = this.store.getRegistrationRequest({
        serverId: registration.registrationRequestId,
      });
      assert(
        localRegistrationRequest,
        `could not find local registration request with server id ${registration.registrationRequestId}`
      );

      const localElection = this.store.getElection({
        serverId: registration.electionId,
      });
      assert(
        localElection,
        `could not find local election with server id ${registration.electionId}`
      );

      const registrationId = this.store.createRegistration({
        id:
          registration.machineId === VX_MACHINE_ID
            ? registration.clientId
            : ClientId(),
        serverId: registration.serverId,
        clientId: registration.clientId,
        machineId: registration.machineId,
        registrationRequestId: localRegistrationRequest.id,
        electionId: localElection.id,
        ballotStyleId: registration.ballotStyleId,
        precinctId: registration.precinctId,
      });

      debug('created or replaced registration %s', registrationId);
    }

    for (const printedBallot of output.printedBallots) {
      const localRegistration = this.store.getRegistration({
        serverId: printedBallot.registrationId,
      });

      assert(
        localRegistration,
        `could not find local registration with server id ${printedBallot.registrationId}`
      );

      const ballotId = this.store.createPrintedBallot({
        id:
          printedBallot.machineId === VX_MACHINE_ID
            ? printedBallot.clientId
            : ClientId(),
        serverId: printedBallot.serverId,
        clientId: printedBallot.clientId,
        machineId: printedBallot.machineId,
        registrationId: localRegistration.clientId,
        castVoteRecord: printedBallot.castVoteRecord,
      });

      debug('created or replaced ballot %s', ballotId);
    }
  }

  private updateServerSyncAttempt({
    syncAttemptId,
    input,
    output,
  }: {
    syncAttemptId: ClientId;
    input: RaveServerSyncInput;
    output: RaveServerSyncOutput;
  }): void {
    const sentParts = describeSyncInputOrOutput(input);
    const receivedParts = describeSyncInputOrOutput(output);

    this.store.updateServerSyncAttempt({
      id: syncAttemptId,
      status: 'success',
      statusMessage: `SENT: ${
        sentParts.length === 0 ? 'nothing' : sentParts.join(', ')
      }\nRECEIVED: ${
        receivedParts.length === 0 ? 'nothing' : receivedParts.join(', ')
      }`,
    });
  }
}

export class MockRaveServerClient implements RaveServerClient {
  constructor(private readonly store: Store) {}

  async sync({
    authStatus,
  }: {
    authStatus?: AuthStatus;
  } = {}): Promise<void> {
    await Promise.resolve();

    const user =
      authStatus?.status === 'logged_in' &&
      authStatus.user.role === 'rave_voter'
        ? authStatus.user
        : null;
    assert(!authStatus || user, 'not logged in as voter');
    const creator = user?.commonAccessCardId ?? 'system';

    const id = this.store.createServerSyncAttempt({
      creator,
      trigger: user ? 'manual' : 'scheduled',
      initialStatusMessage: 'Syncing…',
    });

    setTimeout(() => {
      this.store.updateServerSyncAttempt({
        id,
        status: 'success',
        statusMessage: `SENT: nothing\nRECEIVED: nothing`,
      });
    }, 1000);
  }
}
