import {
  InsertedSmartCardAuthApi,
  InsertedSmartCardAuthMachineState,
} from '@votingworks/auth';
import { useDevDockRouter } from '@votingworks/dev-dock-backend';
import * as grout from '@votingworks/grout';
import express, { Application } from 'express';
import { Id, InsertedSmartCardAuth, VotesDict } from '@votingworks/types';
import { Optional } from '@votingworks/basics';
import { isDeepStrictEqual } from 'util';
import { DateTime } from 'luxon';
import { Workspace } from './workspace';
import { VoterInfo, VoterRegistrationInfo } from './types/db';
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

  registration?: {
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

    /**
     * Election definition as a JSON string.
     */
    electionData?: string;
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

      const registrations = workspace.store.getVoterRegistrations(voterInfo.id);
      const isRaveAdmin = voterInfo.isAdmin;
      const isRegistered = registrations.length > 0;
      if (!isRegistered) {
        return { status: 'unregistered', isRaveAdmin };
      }

      const allRegistrationsPending = registrations.every(
        (registration) => typeof registration.electionId === 'undefined'
      );
      if (allRegistrationsPending) {
        return { status: 'registration_pending', isRaveAdmin };
      }

      const hasVoted = registrations.some(
        (registration) => registration.votedAt
      );

      return { status: hasVoted ? 'voted' : 'registered', isRaveAdmin };
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

    async getVoterRegistrations(): Promise<VoterRegistrationInfo[]> {
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

      return workspace.store.getVoterRegistrations(voterInfo.id);
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

      const id = workspace.store.createVoterRegistration({
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

    async getElectionDefinition() {
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

      const registrations = workspace.store.getVoterRegistrations(voterInfo.id);
      const registration = registrations.find(
        ({ electionId }) => typeof electionId !== 'undefined'
      );

      if (!registration) {
        return undefined;
      }

      return workspace.store.getElectionDefinitionForVoterRegistration(
        registration.id
      );
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

      const registrations = workspace.store.getVoterRegistrations(voterInfo.id);
      // TODO: Handle multiple registrations
      const registration = registrations[0];

      if (!registration) {
        throw new Error('Not registered');
      }

      workspace.store.recordVotesForVoterRegistration({
        voterRegistrationId: registration.id,
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

      if (input.registration) {
        const registrationId = workspace.store.createVoterRegistration({
          commonAccessCardId,
          givenName: input.registration.givenName ?? 'Rebecca',
          familyName: input.registration.familyName ?? 'Welton',
          addressLine1: input.registration.addressLine1 ?? '123 Main St',
          addressLine2: input.registration.addressLine2,
          city: input.registration.city ?? 'Anytown',
          state: input.registration.state ?? 'CA',
          postalCode: input.registration.postalCode ?? '95959',
          stateId: input.registration.stateId ?? 'B2201793',
        });

        if (input.registration.electionData) {
          const electionId = workspace.store.createElectionDefinition(
            input.registration.electionData.toString()
          );

          workspace.store.setVoterRegistrationElection(
            registrationId,
            electionId
          );
        }
      }

      return { commonAccessCardId };
    },

    async getTestVoterVotes() {
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

      const registrations = workspace.store
        .getVoterRegistrations(voterInfo.id)
        .filter(
          (
            registration
          ): registration is VoterRegistrationInfo & { votedAt: DateTime } =>
            registration.votedAt !== undefined
        )
        .sort((a, b) => b.votedAt.valueOf() - a.votedAt.valueOf());

      const [mostRecentlyVotedRegistration] = registrations;
      if (!mostRecentlyVotedRegistration) {
        throw new Error('No votes found');
      }

      const votes = workspace.store.getVotesForVoterRegistration(
        mostRecentlyVotedRegistration.id
      );

      return votes;
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
