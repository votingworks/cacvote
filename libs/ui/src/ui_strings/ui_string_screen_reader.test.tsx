import { advanceTimersAndPromises, mockOf } from '@votingworks/test-utils';
import userEvent from '@testing-library/user-event';
import { act, screen, waitFor } from '../../test/react_testing_library';
import { newTestContext } from '../../test/test_context';
import { ClipParams, PlayAudioClips } from './play_audio_clips';
import { appStrings } from './app_strings';
import { AudioOnly } from './audio_only';
import { LanguageOverride } from './language_override';
import { Button } from '../button';

jest.mock('./play_audio_clips', (): typeof import('./play_audio_clips') => ({
  ...jest.requireActual('./play_audio_clips'),
  PlayAudioClips: jest.fn(),
}));

function getMockClipOutput(clip: ClipParams) {
  return JSON.stringify(clip);
}

beforeAll(() => {
  jest.useFakeTimers();
});

beforeEach(() => {
  mockOf(PlayAudioClips).mockImplementation((props) => {
    const { clips } = props;

    return (
      <div data-testid="mockClips">
        {clips.map((clip) => (
          <span data-testid="mockClipOutput" key={clip.audioId}>
            {getMockClipOutput(clip)}
          </span>
        ))}
      </div>
    );
  });
});

test('queues up audio for <UiString>s within focus/click event targets', async () => {
  const { getAudioContext, mockApiClient, render } = newTestContext();

  mockApiClient.getUiStringAudioIds.mockImplementation((input) => {
    if (input.languageCode === 'en') {
      return Promise.resolve({
        buttonDone: ['abc'],
        titleBmdReviewScreen: ['cba'],
      });
    }

    if (input.languageCode === 'es') {
      return Promise.resolve({
        instructionsBmdReviewPageNavigation: ['def', '123'],
      });
    }

    return Promise.resolve({});
  });

  render(
    <div>
      <div data-testid="clickTarget">
        <h1>{appStrings.titleBmdReviewScreen()}</h1>
        <AudioOnly>
          <LanguageOverride languageCode="es">
            {appStrings.instructionsBmdReviewPageNavigation()}
          </LanguageOverride>
        </AudioOnly>
      </div>
      <Button data-testid="focusTarget" onPress={() => undefined}>
        {appStrings.buttonDone()}
      </Button>
    </div>
  );

  const clickTarget = await screen.findByTestId('clickTarget');
  act(() => getAudioContext()?.setIsEnabled(true));

  // Should trigger audio on click events:
  act(() => userEvent.click(clickTarget));

  const mockClipOutputs = await screen.findAllByTestId('mockClipOutput');
  expect(mockClipOutputs).toHaveLength(3);
  expect(mockClipOutputs[0]).toHaveTextContent(
    getMockClipOutput({ audioId: 'cba', languageCode: 'en' })
  );
  expect(mockClipOutputs[1]).toHaveTextContent(
    getMockClipOutput({ audioId: 'def', languageCode: 'es' })
  );
  expect(mockClipOutputs[2]).toHaveTextContent(
    getMockClipOutput({ audioId: '123', languageCode: 'es' })
  );

  // Should trigger audio on focus events:
  const focusTarget = screen.getByTestId('focusTarget');
  act(() => {
    focusTarget.dispatchEvent(new Event('focus', { bubbles: true }));
  });
  await waitFor(() =>
    expect(screen.queryAllByTestId('mockClipOutput')).toHaveLength(1)
  );
  expect(screen.getByTestId('mockClipOutput')).toHaveTextContent(
    getMockClipOutput({ audioId: 'abc', languageCode: 'en' })
  );
});

test('resumes paused audio when user switches focus', async () => {
  const { getAudioContext, mockApiClient, render } = newTestContext();

  mockApiClient.getUiStringAudioIds.mockResolvedValue({
    titleBmdReviewScreen: ['abc'],
  });

  render(
    <div data-testid="clickTarget">{appStrings.titleBmdReviewScreen()}</div>
  );

  const clickTarget = await screen.findByTestId('clickTarget');

  act(() => getAudioContext()?.setIsEnabled(true));
  await advanceTimersAndPromises();

  act(() => getAudioContext()?.setIsPaused(true));
  expect(getAudioContext()?.isPaused).toEqual(true);

  act(() => userEvent.click(clickTarget));
  await advanceTimersAndPromises();

  expect(getAudioContext()?.isPaused).toEqual(false);
});

test('clears audio queue on blur', async () => {
  const { getAudioContext, mockApiClient, render } = newTestContext();

  mockApiClient.getUiStringAudioIds.mockResolvedValue({
    titleBmdReviewScreen: ['abc'],
  });

  render(
    <div data-testid="clickTarget">{appStrings.titleBmdReviewScreen()}</div>
  );

  const clickTarget = await screen.findByTestId('clickTarget');
  act(() => getAudioContext()?.setIsEnabled(true));
  act(() => userEvent.click(clickTarget));

  const mockClipOutput = await screen.findByTestId('mockClipOutput');
  expect(mockClipOutput).toHaveTextContent(
    getMockClipOutput({ audioId: 'abc', languageCode: 'en' })
  );

  act(() => {
    clickTarget.dispatchEvent(new Event('blur', { bubbles: true }));
  });

  await waitFor(() =>
    expect(screen.queryByTestId('mockClipOutput')).not.toBeInTheDocument()
  );
});

test('triggers replay when user language is changed', async () => {
  const { getAudioContext, getLanguageContext, mockApiClient, render } =
    newTestContext();

  mockApiClient.getUiStringAudioIds.mockImplementation((input) => {
    if (input.languageCode === 'zh-Hans') {
      return Promise.resolve({
        titleBmdReviewScreen: ['abc'],
      });
    }

    if (input.languageCode === 'es') {
      return Promise.resolve({
        titleBmdReviewScreen: ['def'],
      });
    }

    return Promise.resolve({});
  });

  render(
    <div data-testid="clickTarget">{appStrings.titleBmdReviewScreen()}</div>
  );

  const clickTarget = await screen.findByTestId('clickTarget');
  act(() => {
    getAudioContext()?.setIsEnabled(true);
    getLanguageContext()?.setLanguage('zh-Hans');
  });
  act(() => userEvent.click(clickTarget));

  const mockClipOutput = await screen.findByTestId('mockClipOutput');
  expect(mockClipOutput).toHaveTextContent(
    getMockClipOutput({ audioId: 'abc', languageCode: 'zh-Hans' })
  );

  act(() => getLanguageContext()?.setLanguage('es'));

  const updatedMockClipOutput = await screen.findByTestId('mockClipOutput');
  expect(updatedMockClipOutput).toHaveTextContent(
    getMockClipOutput({ audioId: 'def', languageCode: 'es' })
  );
});

test('is a no-op when audio is disabled', async () => {
  const { getAudioContext, mockApiClient, render } = newTestContext();

  mockApiClient.getUiStringAudioIds.mockResolvedValue({
    titleBmdReviewScreen: ['abc'],
  });

  render(
    <div data-testid="clickTarget">{appStrings.titleBmdReviewScreen()}</div>
  );

  const clickTarget = await screen.findByTestId('clickTarget');
  act(() => getAudioContext()?.setIsEnabled(false));
  act(() => userEvent.click(clickTarget));
  await advanceTimersAndPromises();

  expect(screen.queryByTestId('mockClips')).not.toBeInTheDocument();
});

test('handles missing audio ID data', async () => {
  const { getAudioContext, mockApiClient, render } = newTestContext();

  mockApiClient.getUiStringAudioIds.mockResolvedValue({});

  render(
    <div data-testid="clickTarget">{appStrings.titleBmdReviewScreen()}</div>
  );

  const clickTarget = await screen.findByTestId('clickTarget');
  act(() => getAudioContext()?.setIsEnabled(true));
  act(() => userEvent.click(clickTarget));
  await advanceTimersAndPromises();

  expect(screen.queryByTestId('mockClipOutput')).not.toBeInTheDocument();
  screen.getByTestId('clickTarget');
});
