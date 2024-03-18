import { cac } from '@votingworks/auth';
import {
  err,
  ok,
  Optional,
  Result,
  throwIllegalValue,
} from '@votingworks/basics';
import * as grout from '@votingworks/grout';
import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  unsafeParse,
  VotesDict,
} from '@votingworks/types';
import express, { Application } from 'express';
import { isDeepStrictEqual } from 'util';
import { v4 } from 'uuid';
import { DateTime } from 'luxon';
import { Auth, AuthStatus } from './types/auth';
import { Workspace } from './workspace';
import {
  JurisdictionCode,
  Payload,
  RegistrationRequest,
  RegistrationRequestObjectType,
  SignedObject,
  Uuid,
  UuidSchema,
} from './cacvote-server/types';

export type VoterStatus =
  | 'unregistered'
  | 'registration_pending'
  | 'registered'
  | 'voted';

function buildApi({
  auth,
  workspace: { store },
}: {
  auth: Auth;
  workspace: Workspace;
}) {
  async function getAuthStatus(): Promise<AuthStatus> {
    return await auth.getAuthStatus();
  }

  return grout.createApi({
    getAuthStatus,

    checkPin(input: { pin: string }) {
      return auth.checkPin(input.pin);
    },

    getJurisdictionsCodes() {
      return store.getJurisdictionCodes();
    },

    async getVoterStatus(): Promise<Optional<{ status: VoterStatus }>> {
      const authStatus: AuthStatus = await getAuthStatus();

      if (authStatus.status !== 'has_card') {
        return undefined;
      }

      const { commonAccessCardId } = authStatus.card;

      // TODO: support more than one registration request for a given voter
      const registrationRequest = store
        .forEachRegistrationRequest({ commonAccessCardId })
        .first();

      if (!registrationRequest) {
        return { status: 'unregistered' };
      }

      const registration = store
        .forEachRegistration({
          commonAccessCardId,
          registrationRequestObjectId: registrationRequest.object.getId(),
        })
        .first();

      if (!registration) {
        return { status: 'registration_pending' };
      }

      // TODO: check for a submitted ballot to see if the voter has voted
      return { status: 'registered' };
    },

    async getRegistrationRequests(): Promise<RegistrationRequest[]> {
      const authStatus = await getAuthStatus();

      if (authStatus.status !== 'has_card') {
        return [];
      }

      // TODO: get registration requests for the user
      return [];
    },

    async createVoterRegistrationRequest(input: {
      jurisdictionCode: JurisdictionCode;
      givenName: string;
      familyName: string;
      pin: string;
    }): Promise<
      Result<
        { id: Uuid },
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

      const registrationRequest = new RegistrationRequest(
        authStatus.card.commonAccessCardId,
        input.jurisdictionCode,
        input.givenName,
        input.familyName,
        DateTime.now()
      );

      const payload = Payload.of(
        RegistrationRequestObjectType,
        registrationRequest
      ).toBuffer();
      const generateSignatureResult = await auth.generateSignature(payload, {
        pin: input.pin,
      });

      if (generateSignatureResult.isErr()) {
        const error = generateSignatureResult.err();
        switch (error.type) {
          case 'card_error':
            return err({
              type: 'not_logged_in',
              message: `Card error: ${generateSignatureResult.err().message}`,
            });

          case 'incorrect_pin':
            return err({ type: 'incorrect_pin', message: 'Incorrect PIN' });

          default:
            throwIllegalValue(error);
        }
      }

      const certificates = await auth.getCertificate();
      const objectId = unsafeParse(UuidSchema, v4());
      const object = new SignedObject(
        objectId,
        payload,
        certificates,
        generateSignatureResult.ok()
      );

      (await store.addObject(object)).unsafeUnwrap();

      return ok({ id: objectId });
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
    }): Promise<Result<Uuid, cac.GenerateSignatureError>> {
      // TODO: cast and print the ballot
      throw new Error('Not implemented');
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
