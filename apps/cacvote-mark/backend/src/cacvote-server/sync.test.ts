import { deferred, err } from '@votingworks/basics';
import { mockLogger } from '@votingworks/logging';
import e from 'express';
import { DateTime } from 'luxon';
import {
  MockCacvoteAppBuilder,
  mockCacvoteServer,
} from '../../test/mock_cacvote_server';
import { Store } from '../store';
import { createVerifiedObject } from './mock_object';
import { sync, syncPeriodically } from './sync';
import {
  JournalEntry,
  JurisdictionCode,
  JurisdictionCodeSchema,
  Payload,
  RegistrationRequest,
  RegistrationRequestObjectType,
  Uuid,
  UuidSchema,
} from './types';

test('syncPeriodically', async () => {
  const getJournalEntriesDeferred = deferred<void>();
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .onGetJournalEntries(() => {
        getJournalEntriesDeferred.resolve();
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

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
      .onSessionCreate(() => 'session-id')
      .onGetJournalEntries(() => {
        requestCount += 1;
        if (requestCount >= 4) {
          done.resolve();
        }
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

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
      .onSessionCreate(() => 'session-id')
      .onStatusCheck((res) => {
        res.status(500).end('Internal Server Error');
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

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
      .onSessionCreate(() => 'session-id')
      .onGetJournalEntries((res) => {
        res.status(500).end('Internal Server Error');
        getJournalEntriesDeferred.resolve();
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

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
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .withJournalEntries([])
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

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
  const electionId = Uuid();
  const jurisdictionCode = JurisdictionCodeSchema.parse('st.test-jurisdiction');
  const journalEntry = new JournalEntry(
    journalEntryId,
    objectId,
    electionId,
    jurisdictionCode,
    'objectType',
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .withJournalEntries([journalEntry])
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

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
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder().onSessionCreate(() => 'session-id').build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: 'No objects to push to CACvote Server',
      disposition: 'success',
    })
  );
});

test('sync / createObject success / with objects', async () => {
  const object = await createVerifiedObject(
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
      .onSessionCreate(() => 'session-id')
      .onPostObject((_req, res) => {
        res.status(201).send(object.getId());
      })
      .build()
  );

  const store = Store.memoryStore();
  (await store.addObject(object)).unsafeUnwrap();
  expect(store.getObjectsToPush()).toHaveLength(1);

  const logger = mockLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  expect(logger.log).toHaveBeenCalledWith(
    expect.anything(),
    'system',
    expect.objectContaining({
      message: 'Pushing 1 objects to CACvote Server',
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
  const object = await createVerifiedObject(payload);

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .onPostObject((_req, res) => {
        res.status(500).end('Internal Server Error');
      })
      .build()
  );

  const store = Store.memoryStore();
  (await store.addObject(object)).unsafeUnwrap();
  expect(store.getObjectsToPush()).toHaveLength(1);

  const logger = mockLogger();
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
  const object = await createVerifiedObject(
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
    undefined,
    jurisdictionCode,
    RegistrationRequestObjectType,
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .withJournalEntries([journalEntry])
      .onGetObjectById((req, res) => {
        const requestObjectId = UuidSchema.parse(req.params['id']);
        expect(requestObjectId).toEqual(object.getId());
        res.json(object);
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
  expect(store.getObjectById(object.getId())).toEqual(object);
});

test('sync / delete after initial sync', async () => {
  const jurisdictionCode = 'st.dev-jurisdiction' as JurisdictionCode;
  const object = await createVerifiedObject(
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
  const createJournalEntry = new JournalEntry(
    Uuid(),
    object.getId(),
    undefined,
    jurisdictionCode,
    RegistrationRequestObjectType,
    'create',
    DateTime.now()
  );
  const deleteJournalEntry = new JournalEntry(
    Uuid(),
    object.getId(),
    undefined,
    jurisdictionCode,
    RegistrationRequestObjectType,
    'delete',
    DateTime.now()
  );

  const onGetJournalEntries = jest
    .fn<void, [e.Response]>()
    .mockImplementationOnce((res) => {
      res.json([createJournalEntry]);
    })
    .mockImplementationOnce((res) => {
      res.json([createJournalEntry, deleteJournalEntry]);
    })
    .mockImplementation(() => {
      throw new Error('Unexpected call to getJournalEntries');
    });
  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .onGetJournalEntries(onGetJournalEntries)
      .withJournalEntries([createJournalEntry])
      .onGetObjectById((req, res) => {
        const requestObjectId = UuidSchema.parse(req.params['id']);
        expect(requestObjectId).toEqual(object.getId());
        res.json(object);
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

  // first sync
  await sync(server.client, store, logger);

  expect(onGetJournalEntries).toHaveBeenCalledTimes(1);
  const firstSyncEntries = store.getJournalEntries();
  expect(firstSyncEntries).toEqual([createJournalEntry]);
  expect(store.getObjectById(object.getId())).toEqual(object);

  // second sync
  await sync(server.client, store, logger);

  expect(onGetJournalEntries).toHaveBeenCalledTimes(2);
  const secondSyncEntries = store.getJournalEntries();
  expect(secondSyncEntries).toEqual([createJournalEntry, deleteJournalEntry]);
  expect(store.getObjectById(object.getId())).toBeUndefined();

  // wait for the server to stop
  await server.stop();
});

test('sync / delete before initial sync', async () => {
  const jurisdictionCode = 'st.dev-jurisdiction' as JurisdictionCode;
  const object = await createVerifiedObject(
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
  const createJournalEntry = new JournalEntry(
    Uuid(),
    object.getId(),
    undefined,
    jurisdictionCode,
    RegistrationRequestObjectType,
    'create',
    DateTime.now()
  );
  const deleteJournalEntry = new JournalEntry(
    Uuid(),
    object.getId(),
    undefined,
    jurisdictionCode,
    RegistrationRequestObjectType,
    'delete',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .onGetJournalEntries((res) => {
        res.json([createJournalEntry, deleteJournalEntry]);
      })
      .withJournalEntries([createJournalEntry])
      .onGetObjectById((req, res) => {
        const requestObjectId = UuidSchema.parse(req.params['id']);
        expect(requestObjectId).toEqual(object.getId());
        res.json(object);
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

  await sync(server.client, store, logger);

  const entries = store.getJournalEntries();
  expect(entries).toEqual([createJournalEntry, deleteJournalEntry]);
  expect(store.getObjectById(object.getId())).toBeUndefined();

  // wait for the server to stop
  await server.stop();
});

test('sync / fetch ignores unknown object types', async () => {
  const objectId = Uuid();
  const journalEntry = new JournalEntry(
    Uuid(),
    objectId,
    undefined,
    JurisdictionCodeSchema.parse('st.test-jurisdiction'),
    'UnknownType',
    'create',
    DateTime.now()
  );

  const app = new MockCacvoteAppBuilder()
    .onSessionCreate(() => 'session-id')
    .withJournalEntries([journalEntry])
    .build();
  const server = await mockCacvoteServer(app);

  const store = Store.memoryStore();
  const logger = mockLogger();
  await sync(server.client, store, logger);

  // wait for the server to stop
  await server.stop();

  const entries = store.getJournalEntries();
  expect(entries).toEqual([journalEntry]);
  expect(store.getObjectById(objectId)).toBeUndefined();
});

test('sync / fetch failing to get object', async () => {
  const objectId = Uuid();
  const electionId = Uuid();
  const journalEntry = new JournalEntry(
    Uuid(),
    objectId,
    electionId,
    JurisdictionCodeSchema.parse('st.test-jurisdiction'),
    'Registration',
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .withJournalEntries([journalEntry])
      .onGetObjectById((_req, res) => {
        res.status(500).end('Internal Server Error');
      })
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();
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
  const electionId = Uuid();
  const journalEntry = new JournalEntry(
    Uuid(),
    objectId,
    electionId,
    JurisdictionCodeSchema.parse('st.test-jurisdiction'),
    'Registration',
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .withJournalEntries([journalEntry])
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();
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
  const object = await createVerifiedObject(
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
    Uuid(),
    JurisdictionCodeSchema.parse('st.test-jurisdiction'),
    RegistrationRequestObjectType,
    'create',
    DateTime.now()
  );

  const server = await mockCacvoteServer(
    new MockCacvoteAppBuilder()
      .onSessionCreate(() => 'session-id')
      .withJournalEntries([journalEntry])
      .withObject(object)
      .build()
  );

  const store = Store.memoryStore();
  const logger = mockLogger();

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
