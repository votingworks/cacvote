import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { LogSource, Logger } from '@votingworks/logging';
import { ColorMode, ScreenType, SizeMode } from '@votingworks/types';
import { AppBase, ErrorBoundary, H1, P, Prose } from '@votingworks/ui';
import {
  ApiClient,
  ApiClientContext,
  createApiClient,
  createQueryClient,
} from './api';
import { AppRoot } from './app_root';
import { DisplaySettingsManager } from './components/display_settings_manager';

const DEFAULT_COLOR_MODE: ColorMode = 'contrastMedium';
const DEFAULT_SCREEN_TYPE: ScreenType = 'elo15';
const DEFAULT_SIZE_MODE: SizeMode = 'touchSmall';

export interface Props {
  logger?: Logger;
  apiClient?: ApiClient;
  queryClient?: QueryClient;
}

export function App({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  logger = new Logger(LogSource.VxMarkFrontend),
  /* istanbul ignore next */ apiClient = createApiClient(),
  queryClient = createQueryClient(),
}: Props): JSX.Element {
  return (
    <AppBase
      defaultColorMode={DEFAULT_COLOR_MODE}
      defaultSizeMode={DEFAULT_SIZE_MODE}
      screenType={DEFAULT_SCREEN_TYPE}
    >
      <ErrorBoundary
        errorMessage={
          <Prose textCenter>
            <H1>Something went wrong</H1>
            <P>Ask a poll worker to restart the ballot marking device.</P>
          </Prose>
        }
      >
        <ApiClientContext.Provider value={apiClient}>
          <QueryClientProvider client={queryClient}>
            <DisplaySettingsManager />
            <AppRoot />
          </QueryClientProvider>
        </ApiClientContext.Provider>
      </ErrorBoundary>
    </AppBase>
  );
}
