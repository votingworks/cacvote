import userEvent from '@testing-library/user-event';
import { Logger, LogSource } from '@votingworks/logging';
import { screen, waitFor } from '../test/react_testing_library';

import { RebootToBiosButton } from './reboot_to_bios_button';
import { newTestContext } from '../test/test_context';

const { render, mockApiClient } = newTestContext({ skipUiStringsApi: true });

test('renders as expected.', async () => {
  render(
    <RebootToBiosButton
      logger={
        new Logger(LogSource.VxAdminFrontend, () => Promise.resolve('system'))
      }
    />
  );

  userEvent.click(screen.getByText('Reboot to BIOS'));
  await screen.findByText(/Rebooting/);
  await waitFor(() =>
    expect(mockApiClient.rebootToBios).toHaveBeenCalledTimes(1)
  );
});
