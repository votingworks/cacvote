import { safeParseSystemSettings } from '@votingworks/utils';
import { electionMinimalExhaustiveSampleFixtures } from '@votingworks/fixtures';
import { DEFAULT_SYSTEM_SETTINGS, SystemSettings } from '@votingworks/types';
import { Store } from './store';
import { ClientId } from './types/db';

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
    electionMinimalExhaustiveSampleFixtures.systemSettings.asText()
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
    arePollWorkerCardPinsEnabled: true,
  };

  store.setSystemSettings(systemSettingsWithTrue);
  let settings = store.getSystemSettings();
  expect(settings?.arePollWorkerCardPinsEnabled).toEqual(true);

  store.reset();
  const systemSettingsWithFalse: SystemSettings = {
    ...systemSettingsWithTrue,
    arePollWorkerCardPinsEnabled: false,
  };
  store.setSystemSettings(systemSettingsWithFalse);
  settings = store.getSystemSettings();
  expect(settings?.arePollWorkerCardPinsEnabled).toEqual(false);
});

test('reset clears the database', () => {
  const { electionDefinition } = electionMinimalExhaustiveSampleFixtures;
  const store = Store.memoryStore();

  const electionId = ClientId();
  store.createElection({
    id: electionId,
    election: electionDefinition.electionData,
  });
  expect(store.getElection({ clientId: electionId })).toBeTruthy();
  store.reset();
  expect(store.getElection({ clientId: electionId })).toBeFalsy();
});
