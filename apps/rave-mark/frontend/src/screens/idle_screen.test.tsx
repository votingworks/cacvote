import { renderWithThemes as render } from '@votingworks/ui';
import { IdleScreen } from './idle_screen';

test('renders', () => {
  const { container } = render(<IdleScreen />);
  expect(container).toMatchSnapshot();
});
