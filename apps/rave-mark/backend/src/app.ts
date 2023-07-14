import {
  InsertedSmartCardAuthApi,
  InsertedSmartCardAuthMachineState,
} from '@votingworks/auth';
import { VX_MACHINE_ID, buildCastVoteRecord } from '@votingworks/backend';
import { Optional, assert, find, iter } from '@votingworks/basics';
import { useDevDockRouter } from '@votingworks/dev-dock-backend';
import * as grout from '@votingworks/grout';
import {
  BallotIdSchema,
  BallotStyleId,
  BallotType,
  Id,
  InsertedSmartCardAuth,
  PrecinctId,
  VotesDict,
  unsafeParse,
} from '@votingworks/types';
import express, { Application } from 'express';
import { isDeepStrictEqual } from 'util';
import { IS_INTEGRATION_TEST } from './globals';
import { RaveServerClient } from './rave_server_client';
import { AuthStatus } from './types/auth';
import { ClientId, RegistrationRequest } from './types/db';
import { Workspace } from './workspace';

function constructAuthMachineState(
  workspace: Workspace
): InsertedSmartCardAuthMachineState {
  const systemSettings = workspace.store.getSystemSettings();
  return { ...(systemSettings ?? {}) };
}

export type VoterStatus =
  | 'unregistered'
  | 'registration_pending'
  | 'registered'
  | 'voted';

export interface CreateTestVoterInput {
  /**
   * Whether or not the voter should be an admin.
   */
  isAdmin?: boolean;

  registrationRequest?: {
    /**
     * Voter's given name, i.e. first name.
     */
    givenName?: string;

    /**
     * Voter's family name, i.e. last name.
     */
    familyName?: string;

    /**
     * Voter's address line 1.
     */
    addressLine1?: string;

    /**
     * Voter's address line 2.
     */
    addressLine2?: string;

    /**
     * Voter's city.
     */
    city?: string;

    /**
     * Voter's state.
     */
    state?: string;

    /**
     * Voter's postal code.
     */
    postalCode?: string;

    /**
     * Voter's state ID.
     */
    stateId?: string;
  };

  registration?: {
    /**
     * Election definition as a JSON string.
     */
    electionData?: string;

    /**
     * Precinct ID to register the voter to.
     */
    precinctId?: PrecinctId;

    /**
     * Ballot style ID to register the voter to.
     */
    ballotStyleId?: BallotStyleId;
  };
}

function buildApi({
  auth,
  workspace,
  raveServerClient,
}: {
  auth: InsertedSmartCardAuthApi;
  workspace: Workspace;
  raveServerClient: RaveServerClient;
}) {
  async function getAuthStatus(): Promise<AuthStatus> {
    const authStatus = await auth.getAuthStatus(
      constructAuthMachineState(workspace)
    );
    const isRaveAdmin =
      authStatus.status === 'logged_in' &&
      authStatus.user.role === 'rave_voter' &&
      workspace.store.isAdmin(authStatus.user.commonAccessCardId);

    return {
      ...authStatus,
      isRaveAdmin,
    };
  }

  function assertIsIntegrationTest() {
    if (!IS_INTEGRATION_TEST) {
      throw new Error('This is not an integration test');
    }
  }

  return grout.createApi({
    getAuthStatus,

    checkPin(input: { pin: string }) {
      return auth.checkPin(constructAuthMachineState(workspace), {
        pin: input.pin,
      });
    },

    async getVoterStatus(): Promise<
      Optional<{ status: VoterStatus; isRaveAdmin: boolean }>
    > {
      const authStatus: AuthStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        return undefined;
      }

      const { commonAccessCardId } = authStatus.user;
      const isRaveAdmin = workspace.store.isAdmin(commonAccessCardId);
      const registrations =
        workspace.store.getRegistrations(commonAccessCardId);

      if (registrations.length === 0) {
        const registrationRequests =
          workspace.store.getRegistrationRequests(commonAccessCardId);

        return {
          status:
            registrationRequests.length > 0
              ? 'registration_pending'
              : 'unregistered',
          isRaveAdmin,
        };
      }

      // TODO: support multiple registrations
      const registration = registrations[0];
      assert(registration);
      const selection = workspace.store.getCastVoteRecordForRegistration(
        registration.id
      );

      return { status: selection ? 'voted' : 'registered', isRaveAdmin };
    },

    async getRegistrationRequests(): Promise<RegistrationRequest[]> {
      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        return [];
      }

      return workspace.store.getRegistrationRequests(
        authStatus.user.commonAccessCardId
      );
    },

    async createVoterRegistration(input: {
      givenName: string;
      familyName: string;
      addressLine1: string;
      addressLine2?: string;
      city: string;
      state: string;
      postalCode: string;
      stateId: string;
    }): Promise<{ id: Id }> {
      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        throw new Error('Not logged in');
      }

      const id = ClientId();
      workspace.store.createRegistrationRequest({
        id,
        commonAccessCardId: authStatus.user.commonAccessCardId,
        givenName: input.givenName,
        familyName: input.familyName,
        addressLine1: input.addressLine1,
        addressLine2: input.addressLine2,
        city: input.city,
        state: input.state,
        postalCode: input.postalCode,
        stateId: input.stateId,
      });
      return { id };
    },

    async getElectionConfiguration() {
      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        return undefined;
      }

      const { commonAccessCardId } = authStatus.user;
      const registrations =
        workspace.store.getRegistrations(commonAccessCardId);
      // TODO: Handle multiple registrations
      const registration = registrations[0];

      if (!registration) {
        return undefined;
      }

      const electionDefinition = workspace.store.getRegistrationElection(
        registration.id
      );

      if (!electionDefinition) {
        return undefined;
      }

      return {
        electionDefinition,
        ballotStyleId: registration.ballotStyleId,
        precinctId: registration.precinctId,
      };
    },

    async saveVotes(input: { votes: VotesDict }) {
      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        throw new Error('Not logged in');
      }

      const { commonAccessCardId } = authStatus.user;
      const registrations =
        workspace.store.getRegistrations(commonAccessCardId);
      // TODO: Handle multiple registrations
      const registration = registrations[0];

      if (!registration) {
        throw new Error('Not registered');
      }

      const electionDefinition = workspace.store.getRegistrationElection(
        registration.id
      );

      if (!electionDefinition) {
        throw new Error('no election definition found for registration');
      }

      const ballotId = ClientId();
      const castVoteRecordId = unsafeParse(BallotIdSchema, ballotId);
      const castVoteRecord = buildCastVoteRecord({
        election: electionDefinition.election,
        electionId: electionDefinition.electionHash,
        scannerId: VX_MACHINE_ID,
        // TODO: what should the batch ID be?
        batchId: '',
        castVoteRecordId,
        interpretation: {
          type: 'InterpretedBmdPage',
          metadata: {
            ballotStyleId: registration.ballotStyleId,
            precinctId: registration.precinctId,
            ballotType: BallotType.Absentee,
            electionHash: electionDefinition.electionHash,
            // TODO: support test mode
            isTestMode: false,
          },
          votes: input.votes,
        },
        ballotMarkingMode: 'machine',
      });

      workspace.store.createBallot({
        id: ballotId,
        registrationId: registration.id,
        castVoteRecord,
      });
    },

    async sync() {
      const authStatus = await getAuthStatus();
      assert(
        authStatus.status === 'logged_in' &&
          authStatus.user.role === 'rave_voter' &&
          authStatus.isRaveAdmin,
        'not logged in as admin'
      );

      void raveServerClient.sync({ authStatus });
    },

    getServerSyncAttempts() {
      return workspace.store.getServerSyncAttempts();
    },

    logOut() {
      return auth.logOut(constructAuthMachineState(workspace));
    },

    createTestVoter(input: CreateTestVoterInput) {
      assertIsIntegrationTest();

      function createUniqueCommonAccessCardId(): Id {
        const tenRandomDigits = Math.floor(Math.random() * 1e10).toString();
        return `test-${tenRandomDigits.toString().padStart(10, '0')}`;
      }

      const commonAccessCardId = createUniqueCommonAccessCardId();
      if (input.isAdmin) {
        workspace.store.createAdmin({ commonAccessCardId });
      }

      if (input.registrationRequest || input.registration) {
        const registrationRequestId = ClientId();

        workspace.store.createRegistrationRequest({
          id: registrationRequestId,
          commonAccessCardId,
          givenName: input.registrationRequest?.givenName ?? 'Rebecca',
          familyName: input.registrationRequest?.familyName ?? 'Welton',
          addressLine1:
            input.registrationRequest?.addressLine1 ?? '123 Main St',
          addressLine2: input.registrationRequest?.addressLine2,
          city: input.registrationRequest?.city ?? 'Anytown',
          state: input.registrationRequest?.state ?? 'CA',
          postalCode: input.registrationRequest?.postalCode ?? '95959',
          stateId: input.registrationRequest?.stateId ?? 'B2201793',
        });

        if (input.registration?.electionData) {
          const electionId = ClientId();
          workspace.store.createElection({
            id: electionId,
            election: input.registration.electionData.toString(),
          });
          const electionRecord = workspace.store.getElection({
            clientId: electionId,
          });
          assert(electionRecord);

          const { registration } = input;
          const ballotStyle = registration.ballotStyleId
            ? find(
                electionRecord.electionDefinition.election.ballotStyles,
                ({ id }) => id === registration.ballotStyleId
              )
            : electionRecord.electionDefinition.election.ballotStyles[0];
          assert(ballotStyle);

          const precinctId = registration.precinctId
            ? find(
                ballotStyle.precincts,
                (id) => id === registration.precinctId
              )
            : ballotStyle.precincts[0];
          assert(typeof precinctId === 'string');

          workspace.store.createRegistration({
            id: ClientId(),
            registrationRequestId,
            electionId,
            precinctId,
            ballotStyleId: ballotStyle.id,
          });
        }
      }

      return { commonAccessCardId };
    },

    async getTestVoterCastVoteRecord() {
      assertIsIntegrationTest();

      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        throw new Error('Not logged in');
      }

      const { commonAccessCardId } = authStatus.user;
      const mostRecentVotes = iter(
        workspace.store.getRegistrations(commonAccessCardId)
      )
        .flatMap((registration) => {
          const selection = workspace.store.getCastVoteRecordForRegistration(
            registration.id
          );
          return selection ? [selection] : [];
        })
        .first();

      if (!mostRecentVotes) {
        throw new Error('No votes found');
      }

      return mostRecentVotes;
    },
  });
}

export type Api = ReturnType<typeof buildApi>;

export function buildApp({
  auth,
  workspace,
  raveServerClient,
}: {
  auth: InsertedSmartCardAuthApi;
  workspace: Workspace;
  raveServerClient: RaveServerClient;
}): Application {
  const app: Application = express();
  const api = buildApi({ auth, workspace, raveServerClient });

  app.use('/api/watchAuthStatus', (req, res) => {
    res.set({
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      Connection: 'keep-alive',
    });

    let timeout: NodeJS.Timeout | undefined;
    let lastAuthStatus: InsertedSmartCardAuth.AuthStatus | undefined;

    async function sendUpdate() {
      const authStatus = await api.getAuthStatus();

      if (!isDeepStrictEqual(authStatus, lastAuthStatus)) {
        lastAuthStatus = authStatus;
        res.write(`data: ${grout.serialize(authStatus)}\n\n`);
      }

      timeout = setTimeout(
        sendUpdate,
        10 /* AUTH_STATUS_POLLING_INTERVAL_MS */
      );
    }

    req.on('close', () => {
      clearTimeout(timeout);
      res.end();
    });

    void sendUpdate();
  });

  app.use('/api', grout.buildRouter(api, express));
  useDevDockRouter(app, express);
  return app;
}
