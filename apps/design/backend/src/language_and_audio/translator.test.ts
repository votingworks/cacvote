import { LanguageCode } from '@votingworks/types';

import { MockGoogleCloudTranslationClient } from '../../test/helpers';
import { Store } from '../store';
import { GoogleCloudTranslator } from './translator';

test('GoogleCloudTranslator', async () => {
  const store = Store.memoryStore();
  const translationClient = new MockGoogleCloudTranslationClient();
  const translator = new GoogleCloudTranslator({ store, translationClient });

  let translatedTextArray = await translator.translateText(
    ['Do you like apples?', 'Do you like oranges?'],
    LanguageCode.SPANISH
  );
  expect(translatedTextArray).toEqual([
    `Do you like apples? (in ${LanguageCode.SPANISH})`,
    `Do you like oranges? (in ${LanguageCode.SPANISH})`,
  ]);
  expect(translationClient.translateText).toHaveBeenCalledTimes(1);
  expect(translationClient.translateText).toHaveBeenNthCalledWith(
    1,
    expect.objectContaining({
      contents: ['Do you like apples?', 'Do you like oranges?'],
      targetLanguageCode: LanguageCode.SPANISH,
    })
  );
  translationClient.translateText.mockClear();

  // Expect two cache hits and one cache miss
  translatedTextArray = await translator.translateText(
    ['Do you like apples?', 'Do you like bananas?', 'Do you like oranges?'],
    LanguageCode.SPANISH
  );
  expect(translatedTextArray).toEqual([
    `Do you like apples? (in ${LanguageCode.SPANISH})`,
    `Do you like bananas? (in ${LanguageCode.SPANISH})`,
    `Do you like oranges? (in ${LanguageCode.SPANISH})`,
  ]);
  expect(translationClient.translateText).toHaveBeenCalledTimes(1);
  expect(translationClient.translateText).toHaveBeenNthCalledWith(
    1,
    expect.objectContaining({
      contents: ['Do you like bananas?'],
      targetLanguageCode: LanguageCode.SPANISH,
    })
  );
  translationClient.translateText.mockClear();

  // Expect three cache hits
  translatedTextArray = await translator.translateText(
    ['Do you like apples?', 'Do you like bananas?', 'Do you like oranges?'],
    LanguageCode.SPANISH
  );
  expect(translatedTextArray).toEqual([
    `Do you like apples? (in ${LanguageCode.SPANISH})`,
    `Do you like bananas? (in ${LanguageCode.SPANISH})`,
    `Do you like oranges? (in ${LanguageCode.SPANISH})`,
  ]);
  expect(translationClient.translateText).not.toHaveBeenCalled();

  // Expect no cache hits because, though these strings have been translated before, they haven't
  // been translated to this specific language
  translatedTextArray = await translator.translateText(
    ['Do you like apples?', 'Do you like bananas?', 'Do you like oranges?'],
    LanguageCode.CHINESE_TRADITIONAL
  );
  expect(translatedTextArray).toEqual([
    `Do you like apples? (in ${LanguageCode.CHINESE_TRADITIONAL})`,
    `Do you like bananas? (in ${LanguageCode.CHINESE_TRADITIONAL})`,
    `Do you like oranges? (in ${LanguageCode.CHINESE_TRADITIONAL})`,
  ]);
  expect(translationClient.translateText).toHaveBeenCalledTimes(1);
  expect(translationClient.translateText).toHaveBeenNthCalledWith(
    1,
    expect.objectContaining({
      contents: [
        'Do you like apples?',
        'Do you like bananas?',
        'Do you like oranges?',
      ],
      targetLanguageCode: LanguageCode.CHINESE_TRADITIONAL,
    })
  );
});