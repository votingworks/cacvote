import {
  InsertedSmartCardAuthApi,
  InsertedSmartCardAuthMachineState,
} from '@votingworks/auth';
import { useDevDockRouter } from '@votingworks/dev-dock-backend';
import * as grout from '@votingworks/grout';
import express, { Application } from 'express';
import {
  BallotStyleId,
  Id,
  InsertedSmartCardAuth,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import { Optional, assert, find, iter } from '@votingworks/basics';
import { isDeepStrictEqual } from 'util';
import { Workspace } from './workspace';
import { VoterInfo, VoterRegistrationRequest } from './types/db';
import { IS_INTEGRATION_TEST } from './globals';

function constructAuthMachineState(
  workspace: Workspace
): InsertedSmartCardAuthMachineState {
  const systemSettings = workspace.store.getSystemSettings();
  return { ...(systemSettings ?? {}) };
}

export type AuthStatus = InsertedSmartCardAuth.AuthStatus & {
  isRaveAdmin: boolean;
};

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
}: {
  auth: InsertedSmartCardAuthApi;
  workspace: Workspace;
}) {
  async function getAuthStatus(): Promise<AuthStatus> {
    const authStatus = await auth.getAuthStatus(
      constructAuthMachineState(workspace)
    );
    const isRaveAdmin =
      authStatus.status === 'logged_in' &&
      authStatus.user.role === 'rave_voter' &&
      authStatus.user.commonAccessCardId.startsWith('admin-');
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

      const voterInfo = workspace.store.getVoterInfo(
        authStatus.user.commonAccessCardId
      );
      if (!voterInfo) {
        return { status: 'unregistered', isRaveAdmin: false };
      }

      const isRaveAdmin = voterInfo.isAdmin;
      const registrations = workspace.store.getVoterElectionRegistrations(
        voterInfo.id
      );

      if (registrations.length === 0) {
        const registrationRequests =
          workspace.store.getVoterRegistrationRequests(voterInfo.id);

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
      const selection =
        workspace.store.getVoterSelectionForVoterElectionRegistration(
          registration.id
        );

      return { status: selection ? 'voted' : 'registered', isRaveAdmin };
    },

    async getVoterInfo(): Promise<Optional<VoterInfo>> {
      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        return undefined;
      }

      return workspace.store.getVoterInfo(authStatus.user.commonAccessCardId);
    },

    async getVoterRegistrationRequests(): Promise<VoterRegistrationRequest[]> {
      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        return [];
      }

      const voterInfo = workspace.store.getVoterInfo(
        authStatus.user.commonAccessCardId
      );

      if (!voterInfo) {
        return [];
      }

      return workspace.store.getVoterRegistrationRequests(voterInfo.id);
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

      const id = workspace.store.createVoterRegistrationRequest({
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

      const voterInfo = workspace.store.getVoterInfo(
        authStatus.user.commonAccessCardId
      );

      if (!voterInfo) {
        return undefined;
      }

      const registrations = workspace.store.getVoterElectionRegistrations(
        voterInfo.id
      );
      // TODO: Handle multiple registrations
      const registration = registrations[0];

      if (!registration) {
        return undefined;
      }

      const electionDefinition =
        workspace.store.getElectionDefinitionForVoterRegistration(
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

      const voterInfo = workspace.store.getVoterInfo(
        authStatus.user.commonAccessCardId
      );

      if (!voterInfo) {
        throw new Error('Not registered');
      }

      const registrations = workspace.store.getVoterElectionRegistrations(
        voterInfo.id
      );
      // TODO: Handle multiple registrations
      const registration = registrations[0];

      if (!registration) {
        throw new Error('Not registered');
      }

      workspace.store.createVoterSelectionForVoterElectionRegistration({
        voterId: voterInfo.id,
        voterElectionRegistrationId: registration.id,
        votes: input.votes,
      });
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
      const voterInfo =
        workspace.store.getOrCreateVoterInfo(commonAccessCardId);
      workspace.store.setVoterIsAdmin(voterInfo.id, input.isAdmin ?? false);

      if (input.registrationRequest || input.registration) {
        const voterRegistrationRequestId =
          workspace.store.createVoterRegistrationRequest({
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
          const electionId = workspace.store.createElectionDefinition(
            input.registration.electionData.toString()
          );
          const electionDefinition =
            workspace.store.getElectionDefinition(electionId);
          assert(electionDefinition);

          const { registration } = input;

          const ballotStyle = registration.ballotStyleId
            ? find(
                electionDefinition.election.ballotStyles,
                ({ id }) => id === registration.ballotStyleId
              )
            : electionDefinition.election.ballotStyles[0];
          assert(ballotStyle);

          const precinctId = registration.precinctId
            ? find(
                ballotStyle.precincts,
                (id) => id === registration.precinctId
              )
            : ballotStyle.precincts[0];
          assert(typeof precinctId === 'string');

          workspace.store.createVoterElectionRegistration({
            voterId: voterInfo.id,
            voterRegistrationRequestId,
            electionId,
            precinctId,
            ballotStyleId: ballotStyle.id,
          });
        }
      }

      return { commonAccessCardId };
    },

    async getTestVoterSelectionVotes() {
      assertIsIntegrationTest();

      const authStatus = await getAuthStatus();

      if (
        authStatus.status !== 'logged_in' ||
        authStatus.user.role !== 'rave_voter'
      ) {
        throw new Error('Not logged in');
      }

      const voterInfo = workspace.store.getVoterInfo(
        authStatus.user.commonAccessCardId
      );

      if (!voterInfo) {
        throw new Error('Not registered');
      }

      const mostRecentVotes = iter(
        workspace.store.getVoterElectionRegistrations(voterInfo.id)
      )
        .flatMap((registration) => {
          const selection =
            workspace.store.getVoterSelectionForVoterElectionRegistration(
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
}: {
  auth: InsertedSmartCardAuthApi;
  workspace: Workspace;
}): Application {
  const app: Application = express();
  const api = buildApi({ auth, workspace });

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
