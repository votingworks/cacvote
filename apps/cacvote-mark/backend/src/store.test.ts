import { electionTwoPartyPrimaryFixtures } from '@votingworks/fixtures';
import {
  DEFAULT_SYSTEM_SETTINGS,
  SystemSettings,
  safeParseSystemSettings,
} from '@votingworks/types';
import { Store } from './store';

// We pause in some of these tests so we need to increase the timeout
jest.setTimeout(20000);

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
