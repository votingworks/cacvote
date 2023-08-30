import { DefaultTheme, ThemeContext } from 'styled-components';
import React from 'react';
import {
  ThemeManagerContext,
  ThemeManagerContextInterface,
} from '@votingworks/ui';
import { VotesDict } from '@votingworks/types';
import { act, render } from '../../test/react_testing_library';
import {
  UseDisplaySettingsManagerParams,
  useDisplaySettingsManager,
} from './use_display_settings_manager';

const DEFAULT_THEME: Partial<DefaultTheme> = {
  colorMode: 'contrastMedium',
  sizeMode: 'm',
};
const ACTIVE_VOTING_SESSION_VOTES: VotesDict = {};
const NEW_VOTING_SESSION_VOTES = undefined;

let currentTheme: DefaultTheme;
let themeManager: ThemeManagerContextInterface;

function TestHookWrapper(props: UseDisplaySettingsManagerParams): null {
  currentTheme = React.useContext(ThemeContext);
  themeManager = React.useContext(ThemeManagerContext);

  useDisplaySettingsManager(props);

  return null;
}

test('Resets theme when election official logs in', () => {
  const { rerender } = render(
    <TestHookWrapper isLoggedInAsVoter votes={ACTIVE_VOTING_SESSION_VOTES} />,
    { vxTheme: DEFAULT_THEME }
  );

  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>({
      colorMode: 'contrastMedium',
      sizeMode: 'm',
    })
  );

  // Simulate changing display settings as voter:
  act(() => {
    themeManager.setColorMode('contrastLow');
    themeManager.setSizeMode('xl');
  });
  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>({
      colorMode: 'contrastLow',
      sizeMode: 'xl',
    })
  );

  // Should reset display settings on Election Manager login:
  rerender(
    <TestHookWrapper
      isLoggedInAsVoter={false}
      votes={ACTIVE_VOTING_SESSION_VOTES}
    />
  );
  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>(DEFAULT_THEME)
  );

  // Simulate changing display settings as Election Manager:
  act(() => {
    themeManager.setColorMode('contrastHighDark');
    themeManager.setSizeMode('s');
  });
  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>({
      colorMode: 'contrastHighDark',
      sizeMode: 's',
    })
  );

  // Should return to voter settings on return to voter session:
  rerender(
    <TestHookWrapper isLoggedInAsVoter votes={ACTIVE_VOTING_SESSION_VOTES} />
  );
  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>({
      colorMode: 'contrastLow',
      sizeMode: 'xl',
    })
  );
});

test('Resets theme to default if returning to a new voter session', () => {
  const { rerender } = render(
    <TestHookWrapper isLoggedInAsVoter votes={ACTIVE_VOTING_SESSION_VOTES} />,
    { vxTheme: DEFAULT_THEME }
  );

  // Simulate changing display settings as voter:
  act(() => {
    themeManager.setColorMode('contrastLow');
    themeManager.setSizeMode('xl');
  });

  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>({
      colorMode: 'contrastLow',
      sizeMode: 'xl',
    })
  );

  // Simulate logging in ang changing display settings as Election Manager:
  rerender(
    <TestHookWrapper
      isLoggedInAsVoter={false}
      votes={ACTIVE_VOTING_SESSION_VOTES}
    />
  );
  act(() => {
    themeManager.setColorMode('contrastHighDark');
    themeManager.setSizeMode('s');
  });
  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>({
      colorMode: 'contrastHighDark',
      sizeMode: 's',
    })
  );

  // Should reset to default if voter session has been reset:
  rerender(
    <TestHookWrapper isLoggedInAsVoter votes={NEW_VOTING_SESSION_VOTES} />
  );
  expect(currentTheme).toEqual(
    expect.objectContaining<Partial<DefaultTheme>>(DEFAULT_THEME)
  );
});
