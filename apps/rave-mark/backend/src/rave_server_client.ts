// eslint-disable-next-line max-classes-per-file
import { z } from 'zod';
import { assert } from '@votingworks/basics';
import fetch from 'cross-fetch';
import {
  BallotStyleId,
  BallotStyleIdSchema,
  CVR,
  Id,
  PrecinctId,
  PrecinctIdSchema,
  unsafeParse,
} from '@votingworks/types';
import { inspect } from 'util';
import { DateTime } from 'luxon';
import { Store } from './store';
import { AuthStatus } from './types/auth';
import { ClientId, ClientIdSchema, ServerId, ServerIdSchema } from './types/db';

export interface RaveServerClient {
  sync({ authStatus }: { authStatus: AuthStatus }): Promise<void>;
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

interface BallotInput {
  clientId: ClientId;
  machineId: Id;
  commonAccessCardId: string;
  registrationId: Id;
  castVoteRecord: CVR.CVR;
}

type BallotOutput = BallotInput & {
  serverId: ServerId;
};

const BallotOutputSchema: z.ZodSchema<BallotOutput> = z.object({
  serverId: ServerIdSchema,
  clientId: ClientIdSchema,
  machineId: z.string(),
  commonAccessCardId: z.string(),
  registrationId: ClientIdSchema,
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
  election: string;
  createdAt: DateTime;
}

type ElectionOutput = ElectionInput & {
  serverId: ServerId;
};

const ElectionOutputSchema: z.ZodSchema<ElectionOutput> = z.object({
  serverId: ServerIdSchema,
  clientId: ClientIdSchema,
  election: z.string(),
  createdAt: DateTimeSchema,
});

interface RaveMarkSyncInput {
  lastSyncedRegistrationRequestId?: ServerId;
  lastSyncedRegistrationId?: ServerId;
  lastSyncedElectionId?: ServerId;
  lastSyncedBallotId?: ServerId;
  registrationRequests: RegistrationRequestInput[];
  registrations: RegistrationInput[];
  ballots: BallotInput[];
}

interface RaveMarkSyncOutput {
  admins: AdminOutput[];
  elections: ElectionOutput[];
  registrationRequests: RegistrationRequestOutput[];
  registrations: RegistrationOutput[];
  ballots: BallotOutput[];
}

const RaveMarkSyncOutputSchema: z.ZodSchema<RaveMarkSyncOutput> = z.object({
  admins: z.array(AdminOutputSchema),
  elections: z.array(ElectionOutputSchema),
  registrationRequests: z.array(RegistrationRequestOutputSchema),
  registrations: z.array(RegistrationOutputSchema),
  ballots: z.array(BallotOutputSchema),
});

export class RaveServerClientImpl {
  constructor(private readonly store: Store) {}

  async sync({ authStatus }: { authStatus: AuthStatus }): Promise<void> {
    assert(
      authStatus.status === 'logged_in' &&
        authStatus.user.role === 'rave_voter' &&
        authStatus.isRaveAdmin,
      'not logged in as admin'
    );

    const id = this.store.createServerSyncAttempt({
      creator: authStatus.user.commonAccessCardId,
      trigger: 'manual',
      initialStatusMessage: 'Syncing…',
    });

    const input: RaveMarkSyncInput = {
      lastSyncedRegistrationRequestId:
        this.store.getLastSyncedRegistrationRequestId(),
      lastSyncedRegistrationId: this.store.getLastSyncedRegistrationId(),
      lastSyncedElectionId: this.store.getLastSyncedElectionId(),
      lastSyncedBallotId: this.store.getLastSyncedBallotId(),
      registrationRequests: this.store.getRegistrationRequestsToSync(),
      registrations: [],
      ballots: this.store.getBallotsToSync(),
    };

    try {
      console.log(
        'POSTING TO RAVE MARK SYNC',
        inspect(input, false, Infinity),
        JSON.stringify(input)
      );
      const response = await fetch('http://localhost:8000/api/rave-mark/sync', {
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
      const rawOutput = await response.json();
      console.log(
        'RAVE MARK SYNC RESPONSE',
        inspect(rawOutput, false, Infinity)
      );
      const output = unsafeParse(RaveMarkSyncOutputSchema, rawOutput);
      this.store.updateServerSyncAttempt({
        id,
        status: 'success',
        statusMessage: `Sent: ${input.registrationRequests.length} registrations, ${input.ballots.length} ballots. Received: ${output.registrationRequests.length} registrations, ${output.ballots.length} ballots.`,
      });

      this.store.resetAdmins();
      for (const admin of output.admins) {
        this.store.createAdmin(admin);
      }

      for (const election of output.elections) {
        this.store.createElection(election);
      }

      for (const registrationRequest of output.registrationRequests) {
        this.store.createRegistrationRequest(registrationRequest);
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

        this.store.createRegistration({
          serverId: registration.serverId,
          clientId: registration.clientId,
          machineId: registration.machineId,
          registrationRequestId: localRegistrationRequest.id,
          electionId: localElection.id,
          ballotStyleId: registration.ballotStyleId,
          precinctId: registration.precinctId,
        });
      }
    } catch (error) {
      this.store.updateServerSyncAttempt({
        id,
        status: 'failure',
        statusMessage:
          error instanceof Error ? error.message : `unknown error: ${error}`,
      });
    }
  }
}

export class MockRaveServerClient implements RaveServerClient {
  constructor(private readonly store: Store) {}

  async sync({ authStatus }: { authStatus: AuthStatus }): Promise<void> {
    await Promise.resolve();

    assert(
      authStatus.status === 'logged_in' &&
        authStatus.user.role === 'rave_voter' &&
        authStatus.isRaveAdmin,
      'not logged in as admin'
    );

    const id = this.store.createServerSyncAttempt({
      creator: authStatus.user.commonAccessCardId,
      trigger: 'manual',
      initialStatusMessage: 'Syncing…',
    });

    setTimeout(() => {
      this.store.updateServerSyncAttempt({
        id,
        status: 'success',
        statusMessage: `Sent: 0 registrations, 0 votes. Received: 0 registrations, 0 votes.`,
      });
    }, 1000);
  }
}
