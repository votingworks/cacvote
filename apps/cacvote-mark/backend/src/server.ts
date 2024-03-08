import { cac } from '@votingworks/auth';
import { assertDefined, throwIllegalValue } from '@votingworks/basics';
import { LogEventId, Logger } from '@votingworks/logging';
import { Server } from 'http';
import { buildApp } from './app';
import { Auth } from './types/auth';
import { Workspace } from './workspace';
import { Client } from './cacvote-server/client';
import { CACVOTE_URL } from './globals';

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

function getCacvoteServerClient(): Client {
  if (!CACVOTE_URL) {
    throw new Error('CACVOTE_URL not set');
  }

  return new Client(CACVOTE_URL);
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
  const client = getCacvoteServerClient();

  async function doCacvoteServerSync() {
    try {
      const checkResult = await client.checkStatus();

      if (checkResult.isErr()) {
        await logger.log(LogEventId.ApplicationStartup, 'system', {
          message: `Failed to check status of CACVote Server: ${checkResult.err()}`,
          disposition: 'failure',
        });
      } else {
        const latestJournalEntry = workspace.store.getLatestJournalEntry();

        await logger.log(LogEventId.ApplicationStartup, 'system', {
          message: `Checking for journal entries from CACVote Server since ${
            latestJournalEntry?.getId() ?? 'the beginning of time'
          }`,
        });

        const getEntriesResult = await client.getJournalEntries(
          latestJournalEntry?.getId()
        );

        if (getEntriesResult.isErr()) {
          await logger.log(LogEventId.ApplicationStartup, 'system', {
            message: `Failed to get journal entries from CACVote Server: ${
              getEntriesResult.err().message
            }`,
            disposition: 'failure',
          });
        } else {
          const newEntries = getEntriesResult.ok();
          await logger.log(LogEventId.ApplicationStartup, 'system', {
            message: `Got ${newEntries.length} journal entries from CACVote Server`,
            disposition: 'success',
          });

          workspace.store.addJournalEntries(newEntries);

          await logger.log(LogEventId.ApplicationStartup, 'system', {
            message: 'CACVote Server sync succeeded',
            disposition: 'success',
          });
        }
      }
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
