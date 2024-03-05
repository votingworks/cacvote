import { cac } from '@votingworks/auth';
import { assertDefined, throwIllegalValue } from '@votingworks/basics';
import { LogEventId, Logger } from '@votingworks/logging';
import { Server } from 'http';
import { buildApp } from './app';
import {
  DELETE_RECENTLY_CAST_BALLOTS_MINUTES,
  CACVOTE_URL,
  USE_MOCK_CACVOTE_SERVER,
} from './globals';
import {
  MockRaveServerClient,
  RaveServerClient,
  RaveServerClientImpl,
} from './cacvote_server_client';
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

function getRaveServerClient(workspace: Workspace): RaveServerClient {
  if (USE_MOCK_CACVOTE_SERVER) {
    return new MockRaveServerClient(workspace.store);
  }
  const baseUrl = CACVOTE_URL;

  if (!baseUrl) {
    throw new Error('CACVOTE_URL is not set');
  }

  return new RaveServerClientImpl({
    store: workspace.store,
    baseUrl,
  });
}

/**
 * Starts the server with all the default options.
 */
export function start({ auth, logger, port, workspace }: StartOptions): Server {
  const cacvoteServerClient = getRaveServerClient(workspace);
  const resolvedAuth = auth ?? getDefaultAuth();
  const app = buildApp({
    workspace,
    auth: resolvedAuth,
  });

  async function doRaveServerSync() {
    try {
      await cacvoteServerClient.sync();

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
    setTimeout(doRaveServerSync, 1000 * 5);
  }

  void doRaveServerSync().then(
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

  function deleteRecentlyCastBallots(ageInSeconds: number) {
    try {
      workspace.store.deleteRecentlyCastBallots(ageInSeconds);
    } catch (e) {
      console.error('failed to delete recently cast ballots:', e);
    }

    console.log('===> CLEARED!');

    // run again in 5 seconds
    setTimeout(deleteRecentlyCastBallots, 1000 * 5);
  }

  if (DELETE_RECENTLY_CAST_BALLOTS_MINUTES) {
    void deleteRecentlyCastBallots(DELETE_RECENTLY_CAST_BALLOTS_MINUTES * 60);
  }

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
