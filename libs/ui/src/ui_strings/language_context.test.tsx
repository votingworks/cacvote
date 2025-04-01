import { act } from 'react-dom/test-utils';
import { waitFor } from '../../test/react_testing_library';
import { newTestContext } from '../../test/test_context';
import { TEST_UI_STRING_TRANSLATIONS } from '../../test/test_ui_strings';
import {
  DEFAULT_I18NEXT_NAMESPACE,
  DEFAULT_LANGUAGE_CODE,
} from './language_context';

const { getLanguageContext, mockApiClient, render } = newTestContext();

beforeEach(() => {
  jest.resetAllMocks();

  mockApiClient.getAvailableLanguages.mockResolvedValueOnce(['en', 'zh-Hant']);

  mockApiClient.getUiStrings.mockImplementation(({ languageCode }) =>
    Promise.resolve(TEST_UI_STRING_TRANSLATIONS[languageCode] || null)
  );
});

test('availableLanguages', async () => {
  render(<div>foo</div>);

  await waitFor(() => expect(getLanguageContext()).toBeDefined());
  expect(getLanguageContext()?.availableLanguages).toEqual(['en', 'zh-Hant']);
});

test('setLanguage', async () => {
  render(<div>foo</div>);

  await waitFor(() => expect(getLanguageContext()).toBeDefined());

  expect(getLanguageContext()?.currentLanguageCode).toEqual(
    DEFAULT_LANGUAGE_CODE
  );
  expect(
    getLanguageContext()?.i18next.getResourceBundle(
      DEFAULT_LANGUAGE_CODE,
      DEFAULT_I18NEXT_NAMESPACE
    )
  ).toEqual(TEST_UI_STRING_TRANSLATIONS[DEFAULT_LANGUAGE_CODE]);

  act(() => getLanguageContext()?.setLanguage('zh-Hant'));

  await waitFor(() =>
    expect(getLanguageContext()?.currentLanguageCode).toEqual('zh-Hant')
  );
  expect(
    getLanguageContext()?.i18next.getResourceBundle(
      'zh-Hant',
      DEFAULT_I18NEXT_NAMESPACE
    )
  ).toEqual(TEST_UI_STRING_TRANSLATIONS['zh-Hant']);
});
