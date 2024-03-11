import { Buffer } from 'buffer';
import { fakeLogger } from '@votingworks/logging';
import { deferred } from '@votingworks/basics';
import { v4 } from 'uuid';
import { unsafeParse } from '@votingworks/types';
import { DateTime } from 'luxon';
import { readFile } from 'fs/promises';
import { join } from 'path';
import { sync, syncPeriodically } from './sync';
import { Store } from '../store';
import { mockCacvoteServer } from '../../test/mock_cacvote_server';
import {
  JournalEntry,
  JurisdictionCodeSchema,
  Payload,
  SignedObject,
  UuidSchema,
} from './types';

test('syncPeriodically', async () => {
  const getJournalEntriesDeferred = deferred<void>();
  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('[]');
        getJournalEntriesDeferred.resolve();
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  const logger = fakeLogger();

  const stopSyncing = syncPeriodically(server.client, store, logger, 0);

  // wait for the server to receive the request
  await getJournalEntriesDeferred.promise;

  // stop the sync loop
  await stopSyncing();

  // wait for the server to stop
  await server.stop();
});

test('syncPeriodically loops', async () => {
  let statusCount = 0;
  const done = deferred<void>();
  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        statusCount += 1;
        if (statusCount >= 4) {
          done.resolve();
        }
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('[]');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  const logger = fakeLogger();

  const stopSyncing = syncPeriodically(server.client, store, logger, 0);

  // wait for the sync loop to go a few times
  await done.promise;

  // stop the sync loop
  await stopSyncing();

  // wait for the server to stop
  await server.stop();
});

test('sync / checkStatus failure', async () => {
  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end('Internal Server Error');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  const logger = fakeLogger();

  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/Failed to check status/),
      disposition: 'failure',
    })
  );
});

test('sync / getJournalEntries failure', async () => {
  const getJournalEntriesDeferred = deferred<void>();
  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end('Internal Server Error');
        getJournalEntriesDeferred.resolve();
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  const logger = fakeLogger();

  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/Failed to get journal entries/),
      disposition: 'failure',
    })
  );
});

test('sync / getJournalEntries success / no entries', async () => {
  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('[]');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  const logger = fakeLogger();

  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/Got 0 journal entries/),
      disposition: 'success',
    })
  );
});

test('sync / getJournalEntries success / with entries', async () => {
  const journalEntryId = unsafeParse(UuidSchema, v4());
  const objectId = unsafeParse(UuidSchema, v4());
  const jurisdictionCode = unsafeParse(
    JurisdictionCodeSchema,
    'st.test-jurisdiction'
  );
  const journalEntry = new JournalEntry(
    journalEntryId,
    objectId,
    jurisdictionCode,
    'objectType',
    'create',
    DateTime.now()
  );

  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify([journalEntry]));
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  const logger = fakeLogger();

  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/Got 1 journal entries/),
      disposition: 'success',
    })
  );

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
});

test('sync / createObject success / no objects', async () => {
  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('[]');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  const logger = fakeLogger();

  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: 'No objects to push to CACVote Server',
      disposition: 'success',
    })
  );
});

test('sync / createObject success / with objects', async () => {
  const objectId = unsafeParse(UuidSchema, v4());
  const object = new SignedObject(
    objectId,
    Buffer.from(JSON.stringify(new Payload('objectType', Buffer.of(1, 2, 3)))),
    await readFile(
      join(
        __dirname,
        '../../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem'
      )
    ),
    Buffer.of(7, 8, 9)
  );

  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('[]');
        break;

      case 'POST /api/objects':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(objectId);
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  (await store.addObject(object)).unsafeUnwrap();
  expect(store.getUnsyncedObjects()).toHaveLength(1);

  const logger = fakeLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: 'Pushing 1 objects to CACVote Server',
    })
  );

  expect(store.getUnsyncedObjects()).toHaveLength(0);
});

test('sync / createObject failure', async () => {
  const objectId = unsafeParse(UuidSchema, v4());
  const object = new SignedObject(
    objectId,
    Buffer.from(JSON.stringify(new Payload('objectType', Buffer.of(1, 2, 3)))),
    await readFile(
      join(
        __dirname,
        '../../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem'
      )
    ),
    Buffer.of(7, 8, 9)
  );

  const server = mockCacvoteServer((req, res) => {
    switch (`${req.method} ${req.url}`) {
      case 'GET /api/status':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('{}');
        break;

      case 'GET /api/journal-entries':
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end('[]');
        break;

      case 'POST /api/objects':
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end('Internal Server Error');
        break;

      default:
        throw new Error(`Unexpected request: ${req.url}`);
    }
  });

  const store = Store.memoryStore();
  (await store.addObject(object)).unsafeUnwrap();
  expect(store.getUnsyncedObjects()).toHaveLength(1);

  const logger = fakeLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/Failed to push object/),
      disposition: 'failure',
    })
  );

  expect(store.getUnsyncedObjects()).toHaveLength(1);
});
