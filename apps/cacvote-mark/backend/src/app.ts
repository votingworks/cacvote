import {
  cac,
  cryptography,
  getMachineCertPathAndPrivateKey,
} from '@votingworks/auth';
import { VX_MACHINE_ID, buildCastVoteRecord } from '@votingworks/backend';
import {
  Optional,
  Result,
  asyncResultBlock,
  err,
  ok,
  throwIllegalValue,
} from '@votingworks/basics';
import {
  FujitsuThermalPrinterInterface,
  getFujitsuThermalPrinter,
  PrintResult,
} from '@votingworks/fujitsu-thermal-printer';
import * as grout from '@votingworks/grout';
import { Logger } from '@votingworks/logging';
import {
  BallotIdSchema,
  BallotStyleId,
  BallotType,
  ElectionDefinition,
  PrecinctId,
  VotesDict,
  unsafeParse,
} from '@votingworks/types';
import { Buffer } from 'buffer';
import { execFileSync } from 'child_process';
import { createHash } from 'crypto';
import express, { Application } from 'express';
import { DateTime } from 'luxon';
import { Readable } from 'stream';
import { isDeepStrictEqual } from 'util';
import { z } from 'zod';
import {
  Election,
  ElectionObjectType,
  JurisdictionCode,
  Payload,
  RegistrationRequest,
  SignedObject,
  Uuid,
} from './cacvote-server/types';
import { createEncryptedBallotPayload } from './electionguard';
import { MAILING_LABEL_PRINTER } from './globals';
import * as mailingLabel from './mailing_label';
import { Auth, AuthStatus } from './types/auth';
import { BallotVerificationPayload, SignedBuffer } from './verification';
import { Workspace } from './workspace';

export type VoterStatus =
  | 'unregistered'
  | 'registration_pending'
  | 'registered'
  | 'voted';

function buildApi({
  auth,
  workspace: { store },
  ballotPrinter,
}: {
  auth: Auth;
  workspace: Workspace;
  ballotPrinter: FujitsuThermalPrinterInterface;
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

      const castBallot = store
        .forEachCastBallot({
          commonAccessCardId,
          electionObjectId: registration.registration.getElectionObjectId(),
        })
        .first();

      if (!castBallot) {
        return { status: 'registered' };
      }

      return { status: 'voted' };
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

      const payload =
        Payload.RegistrationRequest(registrationRequest).toBuffer();

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
      const objectId = Uuid();
      const object = new SignedObject(
        objectId,
        undefined,
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

      const registrationInfo = store
        .forEachRegistration({
          commonAccessCardId: authStatus.card.commonAccessCardId,
        })
        .first();

      if (!registrationInfo) {
        return undefined;
      }

      const electionObjectId =
        registrationInfo.registration.getElectionObjectId();
      const electionObject = store.getObjectById(electionObjectId);

      if (!electionObject) {
        throw new Error(`Election not found: ${electionObjectId}`);
      }

      const electionPayloadResult = electionObject.getPayload();

      if (electionPayloadResult.isErr()) {
        throw new Error(electionPayloadResult.err().message);
      }

      const electionPayload = electionPayloadResult.ok();
      const election = electionPayload.getData();

      if (!(election instanceof Election)) {
        throw new Error(
          `Expected 'Election' but was ${electionPayload.getObjectType()}`
        );
      }

      return {
        electionDefinition: election.getElectionDefinition(),
        ballotStyleId: registrationInfo.registration.getBallotStyleId(),
        precinctId: registrationInfo.registration.getPrecinctId(),
      };
    },

    castBallot(input: {
      votes: VotesDict;
      serialNumber: number;
      pin: string;
    }): Promise<
      Result<
        { id: Uuid },
        cac.GenerateSignatureError | SyntaxError | z.ZodError
      >
    > {
      return asyncResultBlock(async (bail) => {
        const authStatus = await getAuthStatus();

        if (authStatus.status !== 'has_card') {
          throw new Error('Not logged in');
        }

        const { commonAccessCardId } = authStatus.card;
        // TODO: Handle multiple registrations
        const registration = store
          .forEachRegistration({ commonAccessCardId })
          .first();

        if (!registration) {
          throw new Error('Not registered');
        }

        const electionObjectId =
          registration.registration.getElectionObjectId();
        const electionObject = store.getObjectById(electionObjectId);

        if (!electionObject) {
          throw new Error(`Election not found: ${electionObjectId}`);
        }

        const electionPayload = electionObject
          .getPayloadAsObjectType(ElectionObjectType)
          .okOrElse(bail);
        const election = electionPayload.getData();
        const electionDefinition = election.getElectionDefinition();

        const ballotId = `${input.serialNumber}`;
        const castVoteRecordId = unsafeParse(BallotIdSchema, ballotId);
        const castVoteRecord = buildCastVoteRecord({
          electionDefinition,
          electionId: electionDefinition.electionHash,
          scannerId: VX_MACHINE_ID,
          // TODO: what should the batch ID be?
          batchId: '',
          castVoteRecordId,
          interpretation: {
            type: 'InterpretedBmdPage',
            metadata: {
              ballotStyleId: registration.registration.getBallotStyleId(),
              precinctId: registration.registration.getPrecinctId(),
              ballotType: BallotType.Absentee,
              electionHash: electionDefinition.electionHash,
              // TODO: support test mode
              isTestMode: false,
            },
            votes: input.votes,
          },
          ballotMarkingMode: 'machine',
        });

        const payload = createEncryptedBallotPayload(
          commonAccessCardId,
          electionPayload,
          registration.registration.getRegistrationRequestObjectId(),
          registration.object.getId(),
          electionObjectId,
          castVoteRecord,
          input.serialNumber
        );

        const signature = (
          await auth.generateSignature(payload.toBuffer(), { pin: input.pin })
        ).okOrElse(bail);
        const commonAccessCardCertificate = await auth.getCertificate();
        const objectId = Uuid();
        const object = new SignedObject(
          objectId,
          electionObjectId,
          payload.toBuffer(),
          commonAccessCardCertificate,
          signature
        );

        (await store.addObject(object)).unsafeUnwrap();

        return ok({ id: objectId });
      });
    },

    async printMailingLabel(input: {
      printMailLabelJobId: Uuid;
      castBallotObjectId: Uuid;
    }) {
      return asyncResultBlock(async (bail) => {
        const castBallotObject = store.getObjectById(input.castBallotObjectId);

        if (!castBallotObject) {
          return err(
            new Error(`Cast ballot not found: ${input.castBallotObjectId}`)
          );
        }

        const castBallotPayload = castBallotObject
          .getPayloadAsObjectType('CastBallot')
          .okOrElse(bail);
        const castBallot = castBallotPayload.getData();

        const electionObject = store.getObjectById(
          castBallot.getElectionObjectId()
        );

        if (!electionObject) {
          return err(
            new Error(`Election not found: ${castBallot.getElectionObjectId()}`)
          );
        }

        const addedPrintMailLabelJob = store.addPrintMailLabelJob({
          printMailLabelJobId: input.printMailLabelJobId,
          castBallotObjectId: input.castBallotObjectId,
          electionObjectId: electionObject.getId(),
        });

        if (!addedPrintMailLabelJob) {
          // the print job with the specified ID has already been processed
          return ok();
        }

        const electionPayload = electionObject
          .getPayloadAsObjectType(ElectionObjectType)
          .okOrElse(bail);

        const election = electionPayload.getData();

        const signatureHash = createHash('sha256')
          .update(castBallotObject.getSignature())
          .digest();
        const ballotValidationPayload = new BallotVerificationPayload(
          VX_MACHINE_ID,
          castBallot.getCommonAccessCardId(),
          castBallot.getElectionObjectId(),
          signatureHash
        );
        const ballotValidationPayloadBuffer = ballotValidationPayload.encode();
        const machineAuthConfig = getMachineCertPathAndPrivateKey();
        const ballotValidationPayloadSignature = await cryptography.signMessage(
          {
            message: Readable.from(ballotValidationPayloadBuffer),
            signingPrivateKey: machineAuthConfig.privateKey,
          }
        );
        const signedBuffer = new SignedBuffer(
          ballotValidationPayloadBuffer,
          ballotValidationPayloadSignature
        );
        const qrCodeData = Buffer.from(
          signedBuffer.encode().toString('base64')
        );

        const pdf = await mailingLabel.buildPdf({
          mailingAddress: election.getMailingAddress(),
          qrCodeData,
        });

        execFileSync(
          'lpr',
          ['-P', MAILING_LABEL_PRINTER, '-o', 'media=Custom.4.08x6.47in'],
          { input: pdf }
        );

        return ok();
      });
    },

    async printBallotPdf(input: { pdfData: Buffer }): Promise<PrintResult> {
      const status = await ballotPrinter.getStatus();
      if (status.state !== 'idle') {
        return err(status);
      }

      return await ballotPrinter.print(input.pdfData);
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
  logger,
}: {
  auth: Auth;
  workspace: Workspace;
  logger: Logger;
}): Application {
  const app: Application = express();
  const ballotPrinter = getFujitsuThermalPrinter(logger);
  const api = buildApi({ auth, workspace, ballotPrinter });

  app.use('/api/status', (_req, res) => {
    res.json({ status: 'ok' });
  });

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
