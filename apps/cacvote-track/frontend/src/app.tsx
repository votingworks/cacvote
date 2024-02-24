import { ColorMode, ScreenType, SizeMode } from '@votingworks/types';
import { AppBase, ErrorBoundary, H3, Main, P, Screen } from '@votingworks/ui';
import { AppRoot } from './app_root';

const DEFAULT_COLOR_MODE: ColorMode = 'contrastMedium';
const DEFAULT_SCREEN_TYPE: ScreenType = 'lenovoThinkpad15';
const DEFAULT_SIZE_MODE: SizeMode = 'touchSmall';

export function App(): JSX.Element {
  return (
    <AppBase
      defaultColorMode={DEFAULT_COLOR_MODE}
      defaultSizeMode={DEFAULT_SIZE_MODE}
      screenType={DEFAULT_SCREEN_TYPE}
    >
      <ErrorBoundary
        errorMessage={
          <Screen>
            <Main centerChild>
              <P align="center">
                <H3>Something went wrong</H3>
              </P>
            </Main>
          </Screen>
        }
      >
        <Screen>
          <Main centerChild padded>
            <img
              src="/votingworks-wordmark-black.svg"
              alt="VotingWorks logo"
              style={{ width: '80vw' }}
            />
            <AppRoot />
          </Main>
        </Screen>
      </ErrorBoundary>
    </AppBase>
  );
}
