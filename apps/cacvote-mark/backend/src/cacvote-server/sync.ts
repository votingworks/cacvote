import { deferred, sleep } from '@votingworks/basics';
import { LogEventId, Logger } from '@votingworks/logging';
import { Client } from './client';
import { Store } from '../store';

async function pullJournalEntries(
  client: Client,
  store: Store,
  logger: Logger
): Promise<void> {
  const latestJournalEntry = store.getLatestJournalEntry();

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

    store.addJournalEntries(newEntries);

    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: 'CACVote Server sync succeeded',
      disposition: 'success',
    });
  }
}

async function pushObjects(
  client: Client,
  store: Store,
  logger: Logger
): Promise<void> {
  const objects = store.getUnsyncedObjects();

  if (objects.length === 0) {
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: 'No objects to push to CACVote Server',
      disposition: 'success',
    });
    return;
  }

  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: `Pushing ${objects.length} objects to CACVote Server`,
  });

  for (const object of objects) {
    const pushResult = await client.createObject(object);

    if (pushResult.isErr()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to push object '${object.getId()}' to CACVote Server: ${pushResult.err()}`,
        disposition: 'failure',
      });
      continue;
    }

    store.markObjectAsSynced(object.getId());
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Pushed object with ID '${object.getId()}' to CACVote Server`,
      disposition: 'success',
    });
  }
}

/**
 * Perform a sync with the CACVote Server now.
 */
export async function sync(
  client: Client,
  store: Store,
  logger: Logger
): Promise<void> {
  try {
    const checkResult = await client.checkStatus();

    if (checkResult.isErr()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to check status of CACVote Server: ${
          checkResult.err().message
        }`,
        disposition: 'failure',
      });
      return;
    }

    await pushObjects(client, store, logger);
    await pullJournalEntries(client, store, logger);
  } catch (err) {
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Failed to sync with CACVote Server: ${err}`,
      disposition: 'failure',
    });
  }
}

const SYNC_INTERVAL = 1000 * 5;

/**
 * Synchronizes with the CACVote Server periodically. Returns a function to stop
 * syncing.
 */
export function syncPeriodically(
  client: Client,
  store: Store,
  logger: Logger,
  interval = SYNC_INTERVAL
): () => Promise<void> {
  const stopped = deferred<void>();
  let stopping = false;

  void (async () => {
    while (!stopping) {
      await sync(client, store, logger);

      if (stopping) {
        break;
      }

      await sleep(interval);
    }

    stopped.resolve();
  })();

  return () => {
    stopping = true;
    return stopped.promise;
  };
}
