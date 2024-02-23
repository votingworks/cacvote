import { renderWithThemes as render } from '@votingworks/ui';
import { NoCardScreen } from './no_card_screen';

test('renders', () => {
  const { container } = render(<NoCardScreen />);
  expect(container).toMatchSnapshot();
});
