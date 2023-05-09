import React from 'react';
import { Router } from 'react-router-dom';
import { createMemoryHistory } from 'history';
import userEvent from '@testing-library/user-event';

import { render, screen } from '../../test/react_testing_library';
import { DisplaySettingsScreen } from './display_settings_screen';

test('returns to previous URL on close', () => {
  const history = createMemoryHistory();
  history.push('/initial-url');
  history.push('/display-settings');

  render(
    <Router history={history}>
      <DisplaySettingsScreen />
    </Router>
  );

  expect(history.location.pathname).toEqual('/display-settings');

  userEvent.click(screen.getButton(/done/i));

  expect(history.location.pathname).toEqual('/initial-url');
});