/* istanbul ignore file - test util */

import {
  ElectionPackage,
  ExtendedElectionDefinition,
  UiStringAudioClips,
  UiStringAudioIdsPackage,
  UiStringsPackage,
} from '@votingworks/types';
import { MockUsbDrive } from '@votingworks/usb-drive';
import { extractCdfUiStrings } from '@votingworks/utils';
import { Result, assertDefined } from '@votingworks/basics';
import { UiStringsStore } from './ui_strings_store';
import { mockElectionPackageFileTree } from '../election_package/test_utils';

type MockUsbDriveLike = Pick<MockUsbDrive, 'insertUsbDrive'>;

/** Test context for {@link runUiStringMachineConfigurationTests}. */
export interface UiStringConfigTestContext {
  electionPackage: ExtendedElectionDefinition;
  getMockUsbDrive(): MockUsbDriveLike;
  runConfigureMachine(): Promise<Result<unknown, unknown>>;
  store: UiStringsStore;
}

/**
 * Tests the loading of strings translations and audio into the store when
 * configuring a machine from a USB election package.
 */
export function runUiStringMachineConfigurationTests(
  context: UiStringConfigTestContext
): void {
  const { electionPackage, getMockUsbDrive, runConfigureMachine, store } =
    context;
  const { cdfElection, electionDefinition } = electionPackage;
  const expectedElectionStrings = extractCdfUiStrings(
    assertDefined(cdfElection)
  );

  async function doTestConfigure(usbElectionPackage: ElectionPackage) {
    getMockUsbDrive().insertUsbDrive(
      await mockElectionPackageFileTree(usbElectionPackage)
    );

    const result = await runConfigureMachine();
    expect(result.err()).toBeUndefined();
  }

  test('loads all available UI strings', async () => {
    const appStrings: UiStringsPackage = {
      en: { foo: 'bar', deeply: { nested: 'value' } },
      es: { foo: 'bar_es', deeply: { nested: 'value_es' } },
    };

    await doTestConfigure({ electionDefinition, uiStrings: appStrings });

    expect(store.getLanguages().sort()).toEqual(['en', 'es'].sort());
    expect(store.getUiStrings('en')).toEqual({
      ...assertDefined(appStrings['en']),
      ...assertDefined(expectedElectionStrings['en']),
    });
    expect(store.getUiStrings('es')).toEqual(appStrings['es']);
    expect(store.getUiStrings('zh-Hant')).toBeNull();
  });

  test('is a no-op for missing uiStrings package', async () => {
    await doTestConfigure({ electionDefinition });

    expect(store.getLanguages()).toEqual(['en']);
    expect(store.getUiStrings('en')).toEqual(expectedElectionStrings['en']);
    expect(store.getUiStrings('es')).toBeNull();
  });

  test('loads UI string audio IDs for configured languages', async () => {
    const uiStrings: UiStringsPackage = {
      en: { foo: 'bar' },
      es: { foo: 'bar_es' },
    };

    const uiStringAudioIds: UiStringAudioIdsPackage = {
      en: { foo: ['123', 'abc'] },
      es: { foo: ['456', 'def'] },
      'zh-Hant': { foo: ['789', 'fff'] },
    };

    await doTestConfigure({ electionDefinition, uiStrings, uiStringAudioIds });

    expect(store.getLanguages().sort()).toEqual(['en', 'es'].sort());
    expect(store.getUiStringAudioIds('en')).toEqual({
      ...assertDefined(uiStringAudioIds['en']),
    });
    expect(store.getUiStringAudioIds('es')).toEqual({
      ...assertDefined(uiStringAudioIds['es']),
    });
    expect(store.getUiStringAudioIds('zh-Hant')).toBeNull();
  });

  test('is a no-op for missing uiStringAudioIds package', async () => {
    await doTestConfigure({ electionDefinition });

    expect(store.getUiStringAudioIds('en')).toBeNull();
    expect(store.getUiStringAudioIds('es')).toBeNull();
  });

  test('loads UI string audio clips', async () => {
    const uiStrings: UiStringsPackage = {
      en: { foo: 'bar' },
      es: { foo: 'bar_es' },
    };

    const audioClipsEnglish: UiStringAudioClips = [
      { dataBase64: 'ABC==', id: 'en1', languageCode: 'en' },
      { dataBase64: 'BAC==', id: 'en2', languageCode: 'en' },
      { dataBase64: 'CAB==', id: 'dupeId', languageCode: 'en' },
    ];
    const audioClipsSpanish: UiStringAudioClips = [
      { dataBase64: 'DEF==', id: 'es1', languageCode: 'es' },
      { dataBase64: 'EDF==', id: 'es2', languageCode: 'es' },
      { dataBase64: 'FED==', id: 'dupeId', languageCode: 'es' },
    ];
    const audioClipsUnconfiguredLang: UiStringAudioClips = [
      {
        dataBase64: '123==',
        id: 'dupeId',
        languageCode: 'zh-Hans',
      },
    ];

    await doTestConfigure({
      electionDefinition,
      uiStrings,
      uiStringAudioClips: [
        ...audioClipsEnglish,
        ...audioClipsSpanish,
        ...audioClipsUnconfiguredLang,
      ],
    });

    function getSortedClips(input: {
      audioIds: string[];
      languageCode: string;
    }) {
      return [...store.getAudioClips(input)].sort((a, b) =>
        a.id.localeCompare(b.id)
      );
    }

    expect(
      getSortedClips({
        audioIds: ['en2', 'dupeId'],
        languageCode: 'en',
      })
    ).toEqual([
      { dataBase64: 'CAB==', id: 'dupeId', languageCode: 'en' },
      { dataBase64: 'BAC==', id: 'en2', languageCode: 'en' },
    ]);

    expect(
      getSortedClips({
        audioIds: ['es1', 'dupeId'],
        languageCode: 'es',
      })
    ).toEqual([
      { dataBase64: 'FED==', id: 'dupeId', languageCode: 'es' },
      { dataBase64: 'DEF==', id: 'es1', languageCode: 'es' },
    ]);

    expect(
      getSortedClips({
        audioIds: ['dupeId'],
        languageCode: 'zh-Hans',
      })
    ).toEqual([]);
  });
}
