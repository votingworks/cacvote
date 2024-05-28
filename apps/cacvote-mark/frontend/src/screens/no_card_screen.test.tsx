import { renderWithThemes as render } from '@votingworks/ui';
import { WelcomeScreen } from './welcome_screen';

test('renders', () => {
  const { container } = render(<WelcomeScreen />);
  expect(container).toMatchSnapshot();
});
