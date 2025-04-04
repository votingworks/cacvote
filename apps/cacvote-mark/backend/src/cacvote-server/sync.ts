import {
  assert,
  deferred,
  extractErrorMessage,
  sleep,
} from '@votingworks/basics';
import { LogEventId, Logger } from '@votingworks/logging';
import { ClientApi } from './client';
import { Store } from '../store';
import { CAC_ROOT_CA_CERTS, MACHINE_CERT, VX_CA_CERT } from '../globals';

async function pullJournalEntries(
  client: ClientApi,
  store: Store,
  logger: Logger
): Promise<void> {
  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: 'Pulling journal entries from CACvote Server',
  });

  const latestJournalEntry = store.getLatestJournalEntry();

  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: `Checking for journal entries from CACvote Server since ${
      latestJournalEntry?.getId() ?? 'the beginning of time'
    }`,
  });

  const getEntriesResult = await client.getJournalEntries(
    latestJournalEntry?.getId()
  );

  if (getEntriesResult.isErr()) {
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Failed to get journal entries from CACvote Server: ${
        getEntriesResult.err().message
      }`,
      disposition: 'failure',
    });
  } else {
    const newEntries = getEntriesResult.ok();
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Got ${newEntries.length} journal entries from CACvote Server`,
      disposition: 'success',
    });

    store.addJournalEntries(newEntries);

    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: 'Successfully pulled journal entries from CACvote Server',
      disposition: 'success',
    });
  }
}

async function pushObjects(
  client: ClientApi,
  store: Store,
  logger: Logger
): Promise<void> {
  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: 'Pushing objects to CACvote Server',
  });

  const objects = store.getObjectsToPush();

  if (objects.length === 0) {
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: 'No objects to push to CACvote Server',
      disposition: 'success',
    });
    return;
  }

  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: `Pushing ${objects.length} objects to CACvote Server`,
  });

  for (const object of objects) {
    const pushResult = await client.createObject(object);

    if (pushResult.isErr()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to push object '${object.getId()}' to CACvote Server: ${
          pushResult.err().message
        }`,
        disposition: 'failure',
      });
      continue;
    }

    store.markObjectAsSynced(object.getId());
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Pushed object with ID '${object.getId()}' to CACvote Server`,
      disposition: 'success',
    });
  }

  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: 'Finished pushing objects to CACvote Server',
  });
}

async function pullObjects(
  client: ClientApi,
  store: Store,
  logger: Logger
): Promise<void> {
  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: 'Pulling objects from CACvote Server',
  });

  const journalEntriesForObjectsToFetch =
    store.getJournalEntriesForObjectsToPull();

  for (const journalEntry of journalEntriesForObjectsToFetch) {
    const getObjectResult = await client.getObjectById(
      journalEntry.getObjectId()
    );

    if (getObjectResult.isErr()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to get object with ID '${journalEntry.getObjectId()}' from CACvote Server: ${getObjectResult.err()}`,
        disposition: 'failure',
      });
      continue;
    }

    const object = getObjectResult.ok();

    if (!object) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Object with ID '${journalEntry.getObjectId()}' not found on CACvote Server`,
        disposition: 'failure',
      });
      continue;
    }

    assert(VX_CA_CERT, 'VX_CA_CERT not set');
    assert(MACHINE_CERT, 'MACHINE_CERT not set');
    assert(CAC_ROOT_CA_CERTS, 'CAC_ROOT_CA_CERTS not set');
    const verifyResult = await object.verify(VX_CA_CERT, CAC_ROOT_CA_CERTS);

    if (verifyResult.isErr()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `An error occurred while verifying with ID '${object.getId()}': ${verifyResult.err()}`,
        disposition: 'failure',
      });
      continue;
    }

    if (!verifyResult.ok()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Object with ID '${object.getId()}' failed verification: signature did not match`,
        disposition: 'failure',
      });
      continue;
    }

    const addObjectResult = await store.addObjectFromServer(object);

    if (addObjectResult.isErr()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to add object with ID '${object.getId()}' to the store: ${addObjectResult.err()}`,
        disposition: 'failure',
      });
      continue;
    }

    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Got object with ID '${object.getId()}' and type '${object
        .getPayload()
        .ok()
        ?.getObjectType()}' from CACvote Server`,
      disposition: 'success',
    });
  }

  await logger.log(LogEventId.ApplicationStartup, 'system', {
    message: 'Finished pulling objects from CACvote Server',
  });
}

/**
 * Perform a sync with the CACvote Server now.
 */
export async function sync(
  client: ClientApi,
  store: Store,
  logger: Logger
): Promise<void> {
  try {
    const checkResult = await client.checkStatus();

    if (checkResult.isErr()) {
      await logger.log(LogEventId.ApplicationStartup, 'system', {
        message: `Failed to check status of CACvote Server: ${
          checkResult.err().message
        }`,
        disposition: 'failure',
      });
      return;
    }

    await pushObjects(client, store, logger);
    await pullJournalEntries(client, store, logger);
    await pullObjects(client, store, logger);
  } catch (err) {
    await logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Failed to sync with CACvote Server: ${extractErrorMessage(
        err
      )}`,
      disposition: 'failure',
    });
  }
}

const SYNC_INTERVAL = 1000 * 5;

/**
 * Synchronizes with the CACvote Server periodically. Returns a function to stop
 * syncing.
 */
export function syncPeriodically(
  client: ClientApi,
  store: Store,
  logger: Logger,
  interval = SYNC_INTERVAL
): () => Promise<void> {
  const stopped = deferred<void>();
  let stopping = false;

  client.enrollMachine().catch((err) => {
    void logger.log(LogEventId.ApplicationStartup, 'system', {
      message: `Failed to enroll machine with CACvote Server: ${extractErrorMessage(
        err
      )}`,
      disposition: 'failure',
    });
  });

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
