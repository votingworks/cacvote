import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import { useLanguageControls } from './use_language_controls';
import { createUiStringsApi } from './ui_strings_api';
import { useCurrentLanguage } from './use_current_language';
import { UiStringsContextProvider } from '../ui_strings';
import { act, renderHook, waitFor } from '../../test/react_testing_library';

test('returns external-facing language context API', async () => {
  const mockApi = createUiStringsApi(() => ({
    getAudioClips: jest.fn(),
    getAvailableLanguages: jest.fn().mockResolvedValue([]),
    getUiStringAudioIds: jest.fn(),
    getUiStrings: jest.fn().mockResolvedValue(null),
  }));

  function TestContextWrapper(props: { children: React.ReactNode }) {
    const { children } = props;

    return (
      <QueryClientProvider client={new QueryClient()}>
        <UiStringsContextProvider api={mockApi} noAudio>
          {children}
        </UiStringsContextProvider>
      </QueryClientProvider>
    );
  }

  const { result } = renderHook(
    () => ({
      currentLanguage: useCurrentLanguage(),
      controls: useLanguageControls(),
    }),
    {
      wrapper: TestContextWrapper,
    }
  );

  await waitFor(() => expect(result.current).toBeTruthy());

  expect(result.current.currentLanguage).toEqual('en');

  act(() => result.current.controls.setLanguage('es'));
  expect(result.current.currentLanguage).toEqual('es');

  act(() => result.current.controls.reset());
  expect(result.current.currentLanguage).toEqual('en');
});

test('returns no-op API when no language context is present', () => {
  const { result } = renderHook(() => ({
    currentLanguage: useCurrentLanguage(),
    controls: useLanguageControls(),
  }));

  expect(result.current.currentLanguage).toEqual('en');

  act(() => result.current.controls.setLanguage('es'));
  expect(result.current.currentLanguage).toEqual('en');

  act(() => result.current.controls.reset());
  expect(result.current.currentLanguage).toEqual('en');
});
