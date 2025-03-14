import { ELECTIONGUARD_CLASSPATH } from '@votingworks/backend';
import { assertDefined, iter, ok } from '@votingworks/basics';
import { electionFamousNames2021Fixtures } from '@votingworks/fixtures';
import { Buffer } from 'buffer';
import { DateTime } from 'luxon';
import { mockLogger } from '@votingworks/logging';
import { JURISDICTION_CODE, createVerifiedObject } from './mock_object';
import {
  CastBallot,
  Election,
  ElectionObjectType,
  JournalEntryStruct,
  JurisdictionCodeSchema,
  Payload,
  RegistrationObjectType,
  RegistrationRequest,
  RegistrationRequestObjectType,
  UuidSchema,
} from './types';
import { UsabilityTestClient } from './usability_test_client';

const jurisdictionCode = JurisdictionCodeSchema.parse('st.test-jurisdiction');

test('empty state', async () => {
  const client = new UsabilityTestClient({ logger: mockLogger() });

  const status = await client.checkStatus();
  expect(status).toEqual(ok());

  const uuid = UuidSchema.parse('00000000-0000-0000-0000-000000000000');
  const object = await client.getObjectById(uuid);
  expect(object).toEqual(ok(undefined));

  const journalEntries = await client.getJournalEntries();
  expect(journalEntries).toEqual(ok([]));

  const journalEntriesSince = await client.getJournalEntries(uuid);
  expect(journalEntriesSince).toEqual(ok([]));
});

test('create object', async () => {
  const object = await createVerifiedObject(
    Payload.RegistrationRequest(
      new RegistrationRequest(
        '0123456789',
        jurisdictionCode,
        'Ford',
        'Prefect',
        DateTime.now()
      )
    )
  );

  const client = new UsabilityTestClient({ logger: mockLogger() });

  // echoes the object ID
  const result = await client.createObject(object);
  expect(result).toEqual(ok(object.getId()));

  // can retrieve the object
  const retrievedObject = await client.getObjectById(object.getId());
  expect(retrievedObject).toEqual(ok(object));

  // creates a journal entry
  const getJournalEntriesResult = await client.getJournalEntries();
  const journalEntries = getJournalEntriesResult.unsafeUnwrap();
  expect(journalEntries.map((e) => e.toJSON())).toEqual<JournalEntryStruct[]>([
    {
      id: expect.any(String),
      objectId: object.getId(),
      electionId: object.getElectionId(),
      jurisdictionCode,
      objectType: RegistrationRequestObjectType,
      action: 'create',
      createdAt: expect.any(String),
    },
  ]);

  // no journal entries since the latest one
  expect(await client.getJournalEntries(journalEntries[0]!.getId())).toEqual(
    ok([])
  );
});

test('auto expire completed voting sessions (ballot only)', async () => {
  const electionObject = await createVerifiedObject(
    Payload.Election(
      new Election(
        JURISDICTION_CODE,
        electionFamousNames2021Fixtures.electionDefinition,
        '123 Main St.\nAnytown, USA',
        Buffer.from('public metadata blob')
      )
    )
  );

  const client = new UsabilityTestClient({ logger: mockLogger() });
  (await client.createObject(electionObject)).unsafeUnwrap();

  const registrationRequestObject = await createVerifiedObject(
    Payload.RegistrationRequest(
      new RegistrationRequest(
        '0123456789',
        jurisdictionCode,
        'Ford',
        'Prefect',
        DateTime.now()
      )
    ),
    { electionId: electionObject.getId() }
  );
  const registrationRequestObjectId = registrationRequestObject.getId();

  (await client.createObject(registrationRequestObject)).unsafeUnwrap();

  const journalEntries = (await client.getJournalEntries()).unsafeUnwrap();
  const registrationId = assertDefined(
    iter(journalEntries)
      .filter((e) => e.getObjectType() === RegistrationObjectType)
      .last()
  ).getObjectId();

  const castBallotObject = await createVerifiedObject(
    Payload.CastBallot(
      new CastBallot(
        '0123456789',
        JURISDICTION_CODE,
        registrationRequestObjectId,
        registrationId,
        electionObject.getId(),
        Buffer.from('ballot data')
      )
    ),
    { electionId: electionObject.getId() }
  );
  const castBallotObjectId = castBallotObject.getId();

  (await client.createObject(castBallotObject)).unsafeUnwrap();

  // this should be a no-op
  client.autoExpireCompletedVotingSessions({
    before: DateTime.now().minus({ days: 1 }),
    expire: 'castBallotOnly',
  });

  expect(await client.getObjectById(registrationRequestObjectId)).not.toEqual(
    ok(undefined)
  );
  expect(await client.getObjectById(registrationId)).not.toEqual(ok(undefined));
  expect(await client.getObjectById(castBallotObjectId)).not.toEqual(
    ok(undefined)
  );

  // this should actually expire the cast ballot
  client.autoExpireCompletedVotingSessions({
    before: DateTime.now().plus({ minutes: 1 }),
    expire: 'castBallotOnly',
  });

  expect(await client.getObjectById(castBallotObjectId)).toEqual(ok(undefined));
  expect(await client.getObjectById(registrationRequestObjectId)).not.toEqual(
    ok(undefined)
  );
  expect(await client.getObjectById(registrationId)).not.toEqual(ok(undefined));
});

test('auto expire completed voting sessions (ballot and registration)', async () => {
  const electionObject = await createVerifiedObject(
    Payload.Election(
      new Election(
        JURISDICTION_CODE,
        electionFamousNames2021Fixtures.electionDefinition,
        '123 Main St.\nAnytown, USA',
        Buffer.from('public metadata blob')
      )
    )
  );

  const client = new UsabilityTestClient({ logger: mockLogger() });
  (await client.createObject(electionObject)).unsafeUnwrap();

  const registrationRequestObject = await createVerifiedObject(
    Payload.RegistrationRequest(
      new RegistrationRequest(
        '0123456789',
        jurisdictionCode,
        'Ford',
        'Prefect',
        DateTime.now()
      )
    ),
    { electionId: electionObject.getId() }
  );
  const registrationRequestObjectId = registrationRequestObject.getId();

  (await client.createObject(registrationRequestObject)).unsafeUnwrap();

  const journalEntries = (await client.getJournalEntries()).unsafeUnwrap();
  const registrationId = assertDefined(
    iter(journalEntries)
      .filter((e) => e.getObjectType() === RegistrationObjectType)
      .last()
  ).getObjectId();

  const castBallotObject = await createVerifiedObject(
    Payload.CastBallot(
      new CastBallot(
        '0123456789',
        JURISDICTION_CODE,
        registrationRequestObjectId,
        registrationId,
        electionObject.getId(),
        Buffer.from('ballot data')
      )
    ),
    { electionId: electionObject.getId() }
  );
  const castBallotObjectId = castBallotObject.getId();

  (await client.createObject(castBallotObject)).unsafeUnwrap();

  // this should be a no-op
  client.autoExpireCompletedVotingSessions({
    before: DateTime.now().minus({ days: 1 }),
    expire: 'castBallotAndRegistration',
  });

  expect(await client.getObjectById(registrationRequestObjectId)).not.toEqual(
    ok(undefined)
  );
  expect(await client.getObjectById(registrationId)).not.toEqual(ok(undefined));
  expect(await client.getObjectById(castBallotObjectId)).not.toEqual(
    ok(undefined)
  );

  // this should actually expire the cast ballot
  client.autoExpireCompletedVotingSessions({
    before: DateTime.now().plus({ minutes: 1 }),
    expire: 'castBallotAndRegistration',
  });

  expect(await client.getObjectById(castBallotObjectId)).toEqual(ok(undefined));
  expect(await client.getObjectById(registrationRequestObjectId)).toEqual(
    ok(undefined)
  );
  expect(await client.getObjectById(registrationId)).toEqual(ok(undefined));
});

const egtest = ELECTIONGUARD_CLASSPATH ? test : test.skip;

egtest('with an election', async () => {
  const client = (
    await UsabilityTestClient.withElection(
      electionFamousNames2021Fixtures.electionDefinition,
      { logger: mockLogger() }
    )
  ).unsafeUnwrap();

  const journalEntries = (await client.getJournalEntries()).unsafeUnwrap();
  const electionId = assertDefined(journalEntries[0]!.getObjectId());
  expect(
    journalEntries.map((e) => e.toJSON())
  ).toContainEqual<JournalEntryStruct>({
    id: expect.any(String),
    objectId: expect.any(String),
    electionId: undefined,
    jurisdictionCode: expect.any(String),
    objectType: ElectionObjectType,
    action: 'create',
    createdAt: expect.any(String),
  });

  const registrationRequestObject = await createVerifiedObject(
    Payload.RegistrationRequest(
      new RegistrationRequest(
        '0123456789',
        jurisdictionCode,
        'Ford',
        'Prefect',
        DateTime.now()
      )
    ),
    { electionId }
  );

  (await client.createObject(registrationRequestObject)).unsafeUnwrap();

  // ensure auto registration works
  expect(
    (await client.getJournalEntries(iter(journalEntries).last()!.getId()))
      .unsafeUnwrap()
      .map((e) => e.toJSON())
  ).toEqual<JournalEntryStruct[]>([
    {
      id: expect.any(String),
      objectId: registrationRequestObject.getId(),
      electionId,
      jurisdictionCode,
      objectType: RegistrationRequestObjectType,
      action: 'create',
      createdAt: expect.any(String),
    },
    {
      id: expect.any(String),
      objectId: expect.any(String),
      electionId,
      jurisdictionCode,
      objectType: RegistrationObjectType,
      action: 'create',
      createdAt: expect.any(String),
    },
  ]);
});
