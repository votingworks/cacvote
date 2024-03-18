import { Buffer } from 'buffer';
import { electionTwoPartyPrimaryFixtures } from '@votingworks/fixtures';
import {
  DEFAULT_SYSTEM_SETTINGS,
  SystemSettings,
  safeParseSystemSettings,
  unsafeParse,
} from '@votingworks/types';
import { v4 } from 'uuid';
import { readFile } from 'fs/promises';
import { join } from 'path';
import { DateTime } from 'luxon';
import {
  JurisdictionCode,
  JurisdictionCodeSchema,
  Payload,
  RegistrationRequest,
  RegistrationRequestObjectType,
  SignedObject,
  UuidSchema,
} from './cacvote-server/types';
import { Store } from './store';

// We pause in some of these tests so we need to increase the timeout
jest.setTimeout(20000);

async function getCertificates(): Promise<Buffer> {
  return await readFile(
    join(
      __dirname,
      '../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem'
    )
  );
}

test('getDbPath', () => {
  const store = Store.memoryStore();
  expect(store.getDbPath()).toEqual(':memory:');
});

test('get/set/delete system settings', () => {
  const store = Store.memoryStore();

  expect(store.getSystemSettings()).toBeUndefined();
  const systemSettings = safeParseSystemSettings(
    electionTwoPartyPrimaryFixtures.systemSettings.asText()
  ).unsafeUnwrap();

  store.setSystemSettings(systemSettings);
  expect(store.getSystemSettings()).toEqual(systemSettings);

  store.deleteSystemSettings();
  expect(store.getSystemSettings()).toBeUndefined();
});

test('setSystemSettings can handle boolean values in input', () => {
  const store = Store.memoryStore();
  const systemSettingsWithTrue: SystemSettings = {
    ...DEFAULT_SYSTEM_SETTINGS,
    auth: {
      ...DEFAULT_SYSTEM_SETTINGS.auth,
      arePollWorkerCardPinsEnabled: true,
    },
  };

  store.setSystemSettings(systemSettingsWithTrue);
  let settings = store.getSystemSettings();
  expect(settings?.auth.arePollWorkerCardPinsEnabled).toEqual(true);

  store.reset();
  const systemSettingsWithFalse: SystemSettings = {
    ...systemSettingsWithTrue,
    auth: {
      ...systemSettingsWithTrue.auth,
      arePollWorkerCardPinsEnabled: false,
    },
  };
  store.setSystemSettings(systemSettingsWithFalse);
  settings = store.getSystemSettings();
  expect(settings?.auth.arePollWorkerCardPinsEnabled).toEqual(false);
});

test('reset clears the database', () => {
  const store = Store.memoryStore();
  store.reset();
});

test('forEachObjectOfType', async () => {
  const store = Store.memoryStore();
  const objectType = RegistrationRequestObjectType;

  const object = new SignedObject(
    unsafeParse(UuidSchema, v4()),
    Payload.of(
      objectType,
      new RegistrationRequest(
        '0123456789',
        'st.dev-jurisdiction' as JurisdictionCode,
        'John',
        'Smith',
        DateTime.now()
      )
    ).toBuffer(),
    await getCertificates(),
    Buffer.from('signature')
  );

  expect(store.forEachObjectOfType(objectType).isEmpty()).toBeTruthy();

  (await store.addObject(object)).unsafeUnwrap();

  expect(store.forEachObjectOfType(objectType).count()).toEqual(1);
  expect(store.forEachObjectOfType(objectType).first()).toEqual(object);
  expect(store.forEachObjectOfType('NonExistent').count()).toEqual(0);
});

test('forEachRegistrationRequest', async () => {
  const store = Store.memoryStore();
  const commonAccessCardId = '1234567890';

  const registrationRequest = new RegistrationRequest(
    commonAccessCardId,
    unsafeParse(JurisdictionCodeSchema, 'st.test-jurisdiction'),
    'Given Name',
    'Family Name',
    DateTime.now()
  );
  const object = new SignedObject(
    unsafeParse(UuidSchema, v4()),
    Payload.of(RegistrationRequestObjectType, registrationRequest).toBuffer(),
    await getCertificates(),
    Buffer.from('signature')
  );

  expect(
    store.forEachRegistrationRequest({ commonAccessCardId }).isEmpty()
  ).toBeTruthy();

  (await store.addObject(object)).unsafeUnwrap();

  expect(
    store.forEachRegistrationRequest({ commonAccessCardId }).count()
  ).toEqual(1);
  expect(
    store.forEachRegistrationRequest({ commonAccessCardId }).first()
  ).toEqual({ object, registrationRequest });
  expect(
    store
      .forEachRegistrationRequest({
        commonAccessCardId: `${commonAccessCardId}1`,
      })
      .isEmpty()
  ).toBeTruthy();
});
