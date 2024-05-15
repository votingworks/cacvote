import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ColorMode, ScreenType, SizeMode } from '@votingworks/types';
import { AppBase, ErrorBoundary, H1, P, Prose } from '@votingworks/ui';
import { createQueryClient } from './api';

import { AppRoot } from './app_root';

const DEFAULT_COLOR_MODE: ColorMode = 'contrastMedium';
const DEFAULT_SCREEN_TYPE: ScreenType = 'elo15';
const DEFAULT_SIZE_MODE: SizeMode = 'touchSmall';

export interface Props {
  queryClient?: QueryClient;
}

export function App({ queryClient = createQueryClient() }: Props): JSX.Element {
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
            <P>Please restart the device.</P>
          </Prose>
        }
      >
        <QueryClientProvider client={queryClient}>
          <AppRoot />
        </QueryClientProvider>
      </ErrorBoundary>
    </AppBase>
  );
}
