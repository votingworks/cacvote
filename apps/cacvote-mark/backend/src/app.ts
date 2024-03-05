import { cac } from '@votingworks/auth';
import { err, ok, Optional, Result } from '@votingworks/basics';
import * as grout from '@votingworks/grout';
import {
  BallotStyleId,
  ElectionDefinition,
  Id,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import express, { Application } from 'express';
import { isDeepStrictEqual } from 'util';
import { Auth, AuthStatus } from './types/auth';
import { ClientId, RegistrationRequest, ServerId } from './types/db';
import { Workspace } from './workspace';

export type VoterStatus =
  | 'unregistered'
  | 'registration_pending'
  | 'registered'
  | 'voted';

export interface CreateTestVoterInput {
  jurisdictionId?: string;

  registrationRequest?: {
    /**
     * Voter's given name, i.e. first name.
     */
    givenName?: string;

    /**
     * Voter's family name, i.e. last name.
     */
    familyName?: string;
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

function buildApi({ auth }: { auth: Auth; workspace: Workspace }) {
  async function getAuthStatus(): Promise<AuthStatus> {
    return await auth.getAuthStatus();
  }

  return grout.createApi({
    getAuthStatus,

    checkPin(input: { pin: string }) {
      return auth.checkPin(input.pin);
    },

    async getVoterStatus(): Promise<Optional<{ status: VoterStatus }>> {
      const authStatus: AuthStatus = await getAuthStatus();

      if (authStatus.status !== 'has_card') {
        return undefined;
      }

      // TODO: get voter status for the user
      return undefined;
    },

    async getRegistrationRequests(): Promise<RegistrationRequest[]> {
      const authStatus = await getAuthStatus();

      if (authStatus.status !== 'has_card') {
        return [];
      }

      // TODO: get registration requests for the user
      return [];
    },

    async createVoterRegistration(input: {
      jurisdictionId: ServerId;
      givenName: string;
      familyName: string;
      pin: string;
    }): Promise<
      Result<
        { id: Id },
        { type: 'not_logged_in' | 'incorrect_pin'; message: string }
      >
    > {
      const authStatus = await getAuthStatus();

      if (authStatus.status !== 'has_card') {
        return err({ type: 'not_logged_in', message: 'Not logged in' });
      }

      if (!(await auth.checkPin(input.pin))) {
        return err({ type: 'incorrect_pin', message: 'Incorrect PIN' });
      }

      const id = ClientId();

      // TODO: create registration request

      return ok({ id });
    },

    async getElectionConfiguration(): Promise<
      Optional<{
        electionDefinition: ElectionDefinition;
        ballotStyleId: BallotStyleId;
        precinctId: PrecinctId;
      }>
    > {
      const authStatus = await getAuthStatus();

      if (authStatus.status !== 'has_card') {
        return undefined;
      }

      // TODO: get election configuration for the user
      return undefined;
    },

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    castBallot(_input: {
      votes: VotesDict;
      pin: string;
    }): Promise<Result<ClientId, cac.GenerateSignatureError>> {
      // TODO: cast and print the ballot
      return Promise.resolve(ok(ClientId()));
    },

    logOut() {
      return auth.logOut();
    },
  });
}

export type Api = ReturnType<typeof buildApi>;

export function buildApp({
  auth,
  workspace,
}: {
  auth: Auth;
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
    let lastAuthStatus: AuthStatus | undefined;

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
  return app;
}
