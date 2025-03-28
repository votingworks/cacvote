import userEvent from '@testing-library/user-event';
import { Logger, LogSource } from '@votingworks/logging';
import { screen, waitFor } from '../test/react_testing_library';
import { newTestContext } from '../test/test_context';
import { PowerDownButton } from './power_down_button';

const { mockApiClient, render } = newTestContext({
  skipUiStringsApi: true,
});

test('renders as expected.', async () => {
  render(
    <PowerDownButton
      logger={
        new Logger(LogSource.VxAdminFrontend, () => Promise.resolve('system'))
      }
      userRole="poll_worker"
    />
  );

  userEvent.click(screen.getByText('Power Down'));
  await screen.findByText(/Powering Down/);
  await waitFor(() => expect(mockApiClient.powerDown).toHaveBeenCalledTimes(1));
});
