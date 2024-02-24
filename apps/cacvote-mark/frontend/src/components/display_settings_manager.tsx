import React from 'react';
import {
  VoterSettingsManagerContext,
  useQueryChangeListener,
} from '@votingworks/ui';
import { DefaultTheme, ThemeContext } from 'styled-components';
import { getAuthStatus } from '../api';

/**
 * Side-effect component for monitoring for auth and voter session changes and
 * resetting/restoring voter display settings as needed.
 */
export function DisplaySettingsManager(): JSX.Element | null {
  const themeManager = React.useContext(VoterSettingsManagerContext);
  const currentTheme = React.useContext(ThemeContext);

  const authStatusQuery = getAuthStatus.useQuery();

  const [voterSessionTheme, setVoterSessionTheme] =
    React.useState<DefaultTheme | null>(null);

  useQueryChangeListener(authStatusQuery, {
    select: (data) => data.status,
    onChange: (newStatus) => {
      // Reset to default theme when election official logs in:
      if (newStatus !== 'no_card') {
        setVoterSessionTheme(currentTheme);
        themeManager.resetThemes();
      }

      // Reset to previous voter settings when election official logs out:
      if (newStatus === 'no_card' && voterSessionTheme) {
        themeManager.setColorMode(voterSessionTheme.colorMode);
        themeManager.setSizeMode(voterSessionTheme.sizeMode);
        setVoterSessionTheme(null);
      }
    },
  });

  return null;
}
