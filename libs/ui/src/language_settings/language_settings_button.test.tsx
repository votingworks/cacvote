import { ElectionStringKey, UiStringsPackage } from '@votingworks/types';
import userEvent from '@testing-library/user-event';
import { newTestContext } from '../../test/test_context';
import { LanguageSettingsButton } from './language_settings_button';
import { act, screen } from '../../test/react_testing_library';

test('displays current language', async () => {
  const { getLanguageContext, mockApiClient, render } = newTestContext();

  mockApiClient.getAvailableLanguages.mockResolvedValue(['en', 'es']);

  const testTranslations: UiStringsPackage = {
    ['en']: { [ElectionStringKey.BALLOT_LANGUAGE]: 'English' },
    ['es']: { [ElectionStringKey.BALLOT_LANGUAGE]: 'Español' },
  };
  mockApiClient.getUiStrings.mockImplementation((input) =>
    Promise.resolve(testTranslations[input.languageCode] || null)
  );

  render(<LanguageSettingsButton onPress={jest.fn()} />);
  await screen.findButton('English');

  act(() => getLanguageContext()?.setLanguage('es'));
  await screen.findButton('Español');
});

test('fires onPress event', async () => {
  const { mockApiClient, render } = newTestContext();

  mockApiClient.getAvailableLanguages.mockResolvedValue(['en', 'es']);

  const onPress = jest.fn();

  render(<LanguageSettingsButton onPress={onPress} />);
  expect(onPress).not.toHaveBeenCalled();

  userEvent.click(await screen.findButton('English'));
  expect(onPress).toHaveBeenCalledTimes(1);
});

test('not rendered in single-language contexts', async () => {
  const { mockApiClient, render } = newTestContext();

  mockApiClient.getAvailableLanguages.mockResolvedValue(['en']);

  render(
    <div>
      <h1>Welcome</h1>
      <LanguageSettingsButton onPress={jest.fn()} />
    </div>
  );
  await screen.findByText('Welcome');

  expect(screen.queryByRole('button')).not.toBeInTheDocument();
});
