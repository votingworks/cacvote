import { QueryClientProvider } from '@tanstack/react-query';
import { AppBase, ErrorBoundary, P } from '@votingworks/ui';
import { BrowserRouter } from 'react-router-dom';
import { createQueryClient } from './api';
import { AppRoot } from './app_root';

export function App(): JSX.Element {
  const queryClient = createQueryClient();

  return (
    <AppBase
      defaultColorMode="desktop"
      defaultSizeMode="desktop"
      screenType="builtIn"
    >
      <ErrorBoundary errorMessage={<P>Something went wrong</P>}>
        <QueryClientProvider client={queryClient}>
          <BrowserRouter>
            <AppRoot />
          </BrowserRouter>
        </QueryClientProvider>
      </ErrorBoundary>
    </AppBase>
  );
}
