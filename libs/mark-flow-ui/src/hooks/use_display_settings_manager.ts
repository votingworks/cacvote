import React from 'react';
import { DefaultTheme, ThemeContext } from 'styled-components';

import { ThemeManagerContext } from '@votingworks/ui';
import { VotesDict } from '@votingworks/types';

export interface UseDisplaySettingsManagerParams {
  isLoggedInAsVoter: boolean;
  votes?: VotesDict;
}

export function useDisplaySettingsManager(
  params: UseDisplaySettingsManagerParams
): void {
  const { isLoggedInAsVoter, votes } = params;

  const wasLoggedInAsVoter = React.useRef<boolean>();

  const themeManager = React.useContext(ThemeManagerContext);
  const currentTheme = React.useContext(ThemeContext);
  const [voterSessionTheme, setVoterSessionTheme] =
    React.useState<DefaultTheme | null>(null);

  React.useEffect(() => {
    const wasPreviouslyLoggedInAsVoter = wasLoggedInAsVoter.current;
    const isVotingSessionActive = !!votes;

    // Reset to default theme when election official logs in, since
    // non-voter-facing screens are not optimised for larger text sizes:
    if (wasPreviouslyLoggedInAsVoter && !isLoggedInAsVoter) {
      setVoterSessionTheme(currentTheme);
      themeManager.resetThemes();
    }

    if (
      !wasPreviouslyLoggedInAsVoter &&
      isLoggedInAsVoter &&
      voterSessionTheme
    ) {
      if (isVotingSessionActive) {
        // Reset to previous display settings for the active voter session when
        // when election official logs out:
        themeManager.setColorMode(voterSessionTheme.colorMode);
        themeManager.setSizeMode(voterSessionTheme.sizeMode);
      } else {
        // [VVSG 2.0 7.1-A] Reset themes to default if this is a new voting
        // session:
        themeManager.resetThemes();
      }
      setVoterSessionTheme(null);
    }

    wasLoggedInAsVoter.current = isLoggedInAsVoter;
  }, [currentTheme, isLoggedInAsVoter, themeManager, voterSessionTheme, votes]);
}
