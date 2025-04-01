import { ElectionStringKey, UiStringsPackage } from '@votingworks/types';
import userEvent from '@testing-library/user-event';
import { newTestContext } from '../../test/test_context';
import { LanguageSettingsScreen } from './language_settings_screen';
import {
  act,
  render as renderWithoutContext,
  screen,
  waitFor,
} from '../../test/react_testing_library';

test('displays all available languages', async () => {
  const { getLanguageContext, mockApiClient, render } = newTestContext();

  mockApiClient.getAvailableLanguages.mockResolvedValue([
    'zh-Hans',
    'en',
    'es',
  ]);

  const testTranslations: UiStringsPackage = {
    'zh-Hans': { [ElectionStringKey.BALLOT_LANGUAGE]: '简体中文' },
    en: { [ElectionStringKey.BALLOT_LANGUAGE]: 'English' },
    es: { [ElectionStringKey.BALLOT_LANGUAGE]: 'Español' },
  };
  mockApiClient.getUiStrings.mockImplementation((input) =>
    Promise.resolve(testTranslations[input.languageCode] || null)
  );

  render(<LanguageSettingsScreen onDone={jest.fn()} />);

  await waitFor(() => expect(getLanguageContext()).toBeDefined());
  act(() => getLanguageContext()?.setLanguage('es'));

  const languageButtons = screen.getAllByRole('radio');
  expect(languageButtons).toHaveLength(3);
  expect(languageButtons[0]).toHaveAccessibleName('English');
  expect(languageButtons[1]).toHaveAccessibleName('简体中文');
  expect(languageButtons[2]).toHaveAccessibleName('Selected: Español');

  userEvent.click(languageButtons[1]);
  await screen.findByRole('radio', {
    checked: true,
    name: 'Selected: 简体中文',
  });
});

test('fires onDone event on "Done" button press', () => {
  const onDone = jest.fn();

  renderWithoutContext(<LanguageSettingsScreen onDone={onDone} />);
  expect(onDone).not.toHaveBeenCalled();

  userEvent.click(screen.getButton('Done'));
  expect(onDone).toHaveBeenCalledTimes(1);
});
