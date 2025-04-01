import userEvent from '@testing-library/user-event';
import { AudioControls } from '@votingworks/types';
import { advancePromises, fakeUseAudioControls } from '@votingworks/test-utils';
import { newTestContext } from '../../test/test_context';
import { KeyboardShortcutHandlers } from './keyboard_shortcut_handlers';
import { act, render, screen, waitFor } from '../../test/react_testing_library';
import { useCurrentLanguage } from '../hooks/use_current_language';

const audioControls: AudioControls = fakeUseAudioControls();

jest.mock(
  '../hooks/use_audio_controls',
  (): typeof import('../hooks/use_audio_controls') => ({
    useAudioControls: () => audioControls,
  })
);

test('Shift+L switches display language', async () => {
  const { mockApiClient, render: renderWithContext } = newTestContext();

  mockApiClient.getAvailableLanguages.mockResolvedValue([
    'zh-Hans',
    'en',
    'es',
  ]);

  let currentLanguage: string | undefined;

  function CurrentLanguageConsumer() {
    currentLanguage = useCurrentLanguage();
    return null;
  }

  renderWithContext(
    <div>
      <KeyboardShortcutHandlers />
      <CurrentLanguageConsumer />
      <span>foo</span>
    </div>
  );

  await waitFor(() => screen.getByText('foo'));

  expect(currentLanguage).toEqual('en');

  await act(() => userEvent.keyboard('L'));
  expect(currentLanguage).toEqual('es');

  await act(() => userEvent.keyboard('L'));
  expect(currentLanguage).toEqual('zh-Hans');

  await act(() => userEvent.keyboard('L'));
  expect(currentLanguage).toEqual('en');

  // Should be a no-op without the `Shift` key modifier:
  await act(() => userEvent.keyboard('l'));
  expect(currentLanguage).toEqual('en');
});

test.each([
  { key: 'M', expectedFnCall: audioControls.toggleEnabled },
  { key: 'R', expectedFnCall: audioControls.replay },
  { key: ',', expectedFnCall: audioControls.decreasePlaybackRate },
  { key: '.', expectedFnCall: audioControls.increasePlaybackRate },
  { key: 'P', expectedFnCall: audioControls.togglePause },
  { key: '-', expectedFnCall: audioControls.decreaseVolume },
  { key: '=', expectedFnCall: audioControls.increaseVolume },
])(
  '"$key" key calls expected audioControls function',
  async ({ key, expectedFnCall }) => {
    render(<KeyboardShortcutHandlers />);

    await act(async () => {
      userEvent.keyboard(key);
      await advancePromises();
    });

    expect(expectedFnCall).toHaveBeenCalled();
  }
);
