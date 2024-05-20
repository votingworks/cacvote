import { renderWithThemes } from '@votingworks/ui';
import { screen } from '@testing-library/react';
import { App } from './app';

test('App renders', () => {
  renderWithThemes(<App />);
  const element = screen.getByText('Scan Mailing Label');
  expect(element).toBeInTheDocument();
});
