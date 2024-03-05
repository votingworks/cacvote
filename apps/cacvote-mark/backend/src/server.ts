import { cac } from '@votingworks/auth';
import { assertDefined, throwIllegalValue } from '@votingworks/basics';
import { LogEventId, Logger } from '@votingworks/logging';
import { Server } from 'http';
import { buildApp } from './app';
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
          const cardDetails = assertDefined(status.cardDetails);
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
export function start({ auth, logger, port, workspace }: StartOptions): Server {
  const resolvedAuth = auth ?? getDefaultAuth();
  const app = buildApp({
    workspace,
    auth: resolvedAuth,
  });

  async function doCacvoteServerSync() {
    try {
      // TODO: sync with CACVote Server

      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: 'CACVote Server sync succeeded',
        disposition: 'success',
      });
    } catch (err) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to sync with CACVote Server: ${err}`,
        disposition: 'failure',
      });
    }

    // run again in 5 seconds
    setTimeout(doCacvoteServerSync, 1000 * 5);
  }

  void doCacvoteServerSync().then(
    () =>
      logger.log(LogEventId.ApplicationStartup, 'system', {
        message: 'Started CACVote Server sync',
        disposition: 'success',
      }),
    (err) =>
      logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to start CACVote Server sync: ${err}`,
        disposition: 'failure',
      })
  );

  return app.listen(
    port,
    /* istanbul ignore next */
    async () => {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `RaveMark backend running at http://localhost:${port}/`,
        disposition: 'success',
      });
    }
  );
}
