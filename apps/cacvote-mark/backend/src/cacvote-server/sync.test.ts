import { deferred, err } from '@votingworks/basics';
import { fakeLogger } from '@votingworks/logging';
import { unsafeParse } from '@votingworks/types';
import { Buffer } from 'buffer';
import { readFile } from 'fs/promises';
import { DateTime } from 'luxon';
import { join } from 'path';
import { cryptography } from '@votingworks/auth';
import { Readable } from 'stream';
import {
  MockCacvoteAppBuilder,
  mockCacvoteServer,
} from '../../test/mock_cacvote_server';
import { Store } from '../store';
import { sync, syncPeriodically } from './sync';
import {
  JournalEntry,
  JurisdictionCode,
  JurisdictionCodeSchema,
  Payload,
  RegistrationRequest,
  RegistrationRequestObjectType,
  SignedObject,
  Uuid,
  UuidSchema,
} from './types';

async function getSigningKeyCertificateAuthority(): Promise<Buffer> {
  return await readFile(
    join(
      __dirname,
      '../../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem'
    )
  );
}

function getSigningKeyPrivateKeyPath(): string {
  return join(
    __dirname,
    '../../../../../libs/auth/certs/dev/vx-admin-private-key.pem'
  );
}

async function getSignedObjectForPayload(
  payload: Payload
): Promise<SignedObject> {
  const payloadBuffer = payload.toBuffer();
  const signature = await cryptography.signMessage({
    message: Readable.from(payloadBuffer),
    signingPrivateKey: {
      source: 'file',
      path: getSigningKeyPrivateKeyPath(),
    },
  });
  return new SignedObject(
    Uuid(),
    payloadBuffer,
    await getSigningKeyCertificateAuthority(),
    signature
  );
}

test('syncPeriodically', async () => {
  const getJournalEntriesDeferred = deferred<void>();
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onGetJournalEntries(() => {
        getJournalEntriesDeferred.resolve();
      })
      .build()
  );

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
  let requestCount = 0;
  const done = deferred<void>();
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onGetJournalEntries(() => {
        requestCount += 1;
        if (requestCount >= 4) {
          done.resolve();
        }
      })
      .build()
  );

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
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onStatusCheck((res) => {
        res.status(500).end('Internal Server Error');
      })
      .build()
  );

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
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onGetJournalEntries((res) => {
        res.status(500).end('Internal Server Error');
        getJournalEntriesDeferred.resolve();
      })
      .build()
  );

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
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder().withJournalEntries([]).build()
  );

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
  const journalEntryId = Uuid();
  const objectId = Uuid();
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

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder().withJournalEntries([journalEntry]).build()
  );

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
  const server = await mockCacvoteServer(new MockCacvoteAppBuilder().build());

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
  const object = await getSignedObjectForPayload(
    Payload.RegistrationRequest(
      new RegistrationRequest(
        '0123456789',
        'st.test-jurisdiction' as JurisdictionCode,
        'John',
        'Smith',
        DateTime.now()
      )
    )
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onPostObject((_req, res) => {
        res.status(201).send(object.getId());
      })
      .build()
  );

  const store = Store.memoryStore();
  (await store.addObject(object)).unsafeUnwrap();
  expect(store.getObjectsToPush()).toHaveLength(1);

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

  expect(store.getObjectsToPush()).toHaveLength(0);
});

test('sync / createObject failure', async () => {
  const payload = Payload.RegistrationRequest(
    new RegistrationRequest(
      '0123456789',
      'st.test-jurisdiction' as JurisdictionCode,
      'John',
      'Smith',
      DateTime.now()
    )
  );
  const object = await getSignedObjectForPayload(payload);

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onPostObject((_req, res) => {
        res.status(500).end('Internal Server Error');
      })
      .build()
  );

  const store = Store.memoryStore();
  (await store.addObject(object)).unsafeUnwrap();
  expect(store.getObjectsToPush()).toHaveLength(1);

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

  expect(store.getObjectsToPush()).toHaveLength(1);
});

test('sync / fetches RegistrationRequest objects', async () => {
  const jurisdictionCode = 'st.dev-jurisdiction' as JurisdictionCode;
  const object = await getSignedObjectForPayload(
    Payload.RegistrationRequest(
      new RegistrationRequest(
        '0123456789',
        jurisdictionCode,
        'John',
        'Smith',
        DateTime.now()
      )
    )
  );
  const journalEntry = new JournalEntry(
    Uuid(),
    object.getId(),
    jurisdictionCode,
    RegistrationRequestObjectType,
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .withJournalEntries([journalEntry])
      .onGetObjectById((req, res) => {
        const requestObjectId = unsafeParse(UuidSchema, req.params['id']);
        expect(requestObjectId).toEqual(object.getId());
        res.json(object);
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = fakeLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
  expect(store.getObjectById(object.getId())).toEqual(object);
});

test('sync / fetch ignores unknown object types', async () => {
  const objectId = Uuid();
  const journalEntry = new JournalEntry(
    Uuid(),
    objectId,
    unsafeParse(JurisdictionCodeSchema, 'st.test-jurisdiction'),
    'UnknownType',
    'create',
    DateTime.now()
  );

  const app = new MockCacvoteAppBuilder()
    .withJournalEntries([journalEntry])
    .build();
  const server = await mockCacvoteServer(app);

  const store = Store.memoryStore();
  const logger = fakeLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
  expect(store.getObjectById(objectId)).toBeUndefined();
});

test('sync / fetch failing to get object', async () => {
  const objectId = Uuid();
  const journalEntry = new JournalEntry(
    Uuid(),
    objectId,
    unsafeParse(JurisdictionCodeSchema, 'st.test-jurisdiction'),
    'Registration',
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .withJournalEntries([journalEntry])
      .onGetObjectById((_req, res) => {
        res.status(500).end('Internal Server Error');
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = fakeLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
  expect(store.getObjectById(objectId)).toBeUndefined();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/Failed to get object/),
      disposition: 'failure',
    })
  );
});

test('sync / fetch object but object does not exist', async () => {
  const objectId = Uuid();
  const journalEntry = new JournalEntry(
    Uuid(),
    objectId,
    unsafeParse(JurisdictionCodeSchema, 'st.test-jurisdiction'),
    'Registration',
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder().withJournalEntries([journalEntry]).build()
  );

  const store = Store.memoryStore();
  const logger = fakeLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
  expect(store.getObjectById(objectId)).toBeUndefined();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/not found/),
      disposition: 'failure',
    })
  );
});

test('sync / fetch object but cannot add to store', async () => {
  const object = await getSignedObjectForPayload(
    Payload.RegistrationRequest(
      new RegistrationRequest(
        '0123456789',
        'st.dev-jurisdiction' as JurisdictionCode,
        'John',
        'Smith',
        DateTime.now()
      )
    )
  );
  const journalEntry = new JournalEntry(
    Uuid(),
    object.getId(),
    unsafeParse(JurisdictionCodeSchema, 'st.test-jurisdiction'),
    'RegistrationRequest',
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .withJournalEntries([journalEntry])
      .withObject(object)
      .build()
  );

  const store = Store.memoryStore();
  const logger = fakeLogger();

  jest
    .spyOn(store, 'addObjectFromServer')
    .mockResolvedValue(err(new SyntaxError('bad object!')));

  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
  expect(store.getObjectById(object.getId())).toBeUndefined();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: expect.stringMatching(/Failed to add object/),
      disposition: 'failure',
    })
  );
});
