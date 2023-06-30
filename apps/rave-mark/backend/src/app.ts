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
import { Workspace } from './workspace';
import { VoterInfo, VoterRegistrationInfo } from './types/db';

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
