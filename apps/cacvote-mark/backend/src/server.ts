import { cac, cryptography } from '@votingworks/auth';
import { VX_MACHINE_ID } from '@votingworks/backend';
import { throwIllegalValue } from '@votingworks/basics';
import { LogEventId, Logger } from '@votingworks/logging';
import { Buffer } from 'buffer';
import { Server } from 'http';
import { Readable } from 'stream';
import { inspect } from 'util';
import { buildApp } from './app';
import { Client } from './cacvote-server/client';
import { syncPeriodically } from './cacvote-server/sync';
import {
  CACVOTE_URL,
  IS_RUNNING_USABILITY_TEST,
  MACHINE_CERT,
  SIGNER,
} from './globals';
import { Auth } from './types/auth';
import { Workspace } from './workspace';

export interface StartOptions {
  auth?: Auth;
  workspace: Workspace;
  logger: Logger;
  port: number;
}

function getDefaultAuth(): Auth {
  const card: cac.CommonAccessCardCompatibleCard = new cac.CommonAccessCard();

  return {
    async checkPin(pin) {
      const result = await card.checkPin(pin);
      return result.response === 'correct';
    },

    async getAuthStatus() {
      const status = await card.getCardStatus();

      switch (status.status) {
        case 'no_card':
        case 'no_card_reader':
        case 'unknown_error':
        case 'card_error':
          return { status: 'no_card' };

        case 'ready': {
          const { cardDetails } = status;

          if (!cardDetails) {
            return { status: 'no_card' };
          }

          return {
            status: 'has_card',
            card: cardDetails,
          };
        }

        /* istanbul ignore next: Compile-time check for completeness */
        default:
          throwIllegalValue(status);
      }
    },

    getCertificate() {
      return card.getCertificate({ objectId: cac.CARD_DOD_CERT.OBJECT_ID });
    },

    generateSignature(message, options) {
      return card.generateSignature(message, {
        privateKeyId: cac.CARD_DOD_CERT.PRIVATE_KEY_ID,
        pin: options.pin,
      });
    },

    async logOut() {
      await card.disconnect();
    },
  };
}

/**
 * Starts the server with all the default options.
 */
export async function start({
  auth,
  logger,
  port,
  workspace,
}: StartOptions): Promise<Server> {
  const resolvedAuth = auth ?? getDefaultAuth();
  const app = await buildApp({
    workspace,
    auth: resolvedAuth,
    logger,
  });

  if (!IS_RUNNING_USABILITY_TEST) {
    if (!CACVOTE_URL) {
      throw new Error('CACVOTE_URL not set');
    }

    if (!MACHINE_CERT) {
      throw new Error('MACHINE_CERT not set');
    }

    if (!SIGNER) {
      throw new Error('SIGNER not set');
    }

    // verify that the SIGNER signs such that the MACHINE_CERT can verify
    const message = Buffer.from('test');
    const messageSignature = await cryptography.signMessage({
      message: Readable.from(message),
      signingPrivateKey: SIGNER,
    });
    const publicKey = await cryptography.extractPublicKeyFromCert(MACHINE_CERT);
    try {
      await cryptography.verifySignature({
        message: Readable.from(message),
        messageSignature,
        publicKey,
      });
    } catch (e) {
      void logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `SIGNER and MACHINE_CERT do not match: ${e}\nPlease check that the SIGNER and MACHINE_CERT are compatible.\nSIGNER: ${inspect(
          SIGNER
        )}\nMACHINE_CERT: ${MACHINE_CERT.toString()}`,
        disposition: 'failure',
      });
      throw e;
    }

    syncPeriodically(
      new Client(logger, CACVOTE_URL, VX_MACHINE_ID, MACHINE_CERT, SIGNER),
      workspace.store,
      logger
    );
  }

  return app.listen(
    port,
    /* istanbul ignore next */
    async () => {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `CACvote Mark backend running at http://localhost:${port}/`,
        disposition: 'success',
      });
    }
  );
}
