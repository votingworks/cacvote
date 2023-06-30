import { render, screen, waitFor } from '@testing-library/react';
import { Buffer } from 'buffer';
import userEvent from '@testing-library/user-event';
import { createMockClient, MockClient } from '@votingworks/grout-test-utils';
import type { Api } from '@votingworks/dev-dock-backend';
import {
  BooleanEnvironmentVariableName,
  getFeatureFlagMock,
} from '@votingworks/utils';
import { fakeKiosk, fakeFileWriter } from '@votingworks/test-utils';
import { CardStatus } from '@votingworks/auth';
import { DevDock } from './dev_dock';

const noCardStatus: CardStatus = {
  status: 'no_card',
};
const featureFlagMock = getFeatureFlagMock();
jest.mock('@votingworks/utils', () => {
  return {
    ...jest.requireActual('@votingworks/utils'),
    isFeatureFlagEnabled: (flag: BooleanEnvironmentVariableName) =>
      featureFlagMock.isEnabled(flag),
  };
});

let mockApiClient: MockClient<Api>;
let mockKiosk!: jest.Mocked<KioskBrowser.Kiosk>;

beforeEach(() => {
  mockApiClient = createMockClient<Api>();
  mockApiClient.getCardStatus.expectCallWith().resolves(noCardStatus);
  mockApiClient.getUsbDriveStatus.expectCallWith().resolves('removed');
  featureFlagMock.enableFeatureFlag(
    BooleanEnvironmentVariableName.ENABLE_DEV_DOCK
  );
  featureFlagMock.enableFeatureFlag(
    BooleanEnvironmentVariableName.USE_MOCK_CARDS
  );
  mockKiosk = fakeKiosk();
  window.kiosk = mockKiosk;
});

afterEach(() => {
  mockApiClient.assertComplete();
  featureFlagMock.resetFeatureFlags();
});

test('renders nothing if dev dock is disabled', () => {
  mockApiClient.getCardStatus.reset();
  mockApiClient.getUsbDriveStatus.reset();
  featureFlagMock.disableFeatureFlag(
    BooleanEnvironmentVariableName.ENABLE_DEV_DOCK
  );
  const { container } = render(<DevDock apiClient={mockApiClient} />);
  expect(container).toBeEmptyDOMElement();
});

test('USB drive controls', async () => {
  render(<DevDock apiClient={mockApiClient} />);
  const usbDriveControl = screen.getByRole('button', {
    name: 'USB Drive',
  });

  mockApiClient.insertUsbDrive.expectCallWith().resolves();
  mockApiClient.getUsbDriveStatus.expectCallWith().resolves('inserted');
  userEvent.click(usbDriveControl);
  // Not easy to test the color change of the button, so we'll just wait for the
  // API call to complete.
  await waitFor(() => mockApiClient.assertComplete());

  mockApiClient.removeUsbDrive.expectCallWith().resolves();
  mockApiClient.getUsbDriveStatus.expectCallWith().resolves('removed');
  userEvent.click(usbDriveControl);
  await waitFor(() => mockApiClient.assertComplete());

  const clearUsbDriveButton = screen.getByRole('button', {
    name: 'Clear',
  });
  mockApiClient.clearUsbDrive.expectCallWith().resolves();
  mockApiClient.getUsbDriveStatus.expectCallWith().resolves('removed');
  userEvent.click(clearUsbDriveButton);
  await waitFor(() => mockApiClient.assertComplete());
});

test('screenshot button', async () => {
  render(<DevDock apiClient={mockApiClient} />);
  const screenshotButton = screen.getByRole('button', {
    name: 'Capture Screenshot',
  });

  const screenshotBuffer = Buffer.of();
  const fileWriter = fakeFileWriter();
  mockKiosk.captureScreenshot.mockResolvedValueOnce(screenshotBuffer);
  mockKiosk.saveAs.mockResolvedValueOnce(fileWriter);
  userEvent.click(screenshotButton);

  await waitFor(() => {
    expect(mockKiosk.captureScreenshot).toHaveBeenCalled();
    expect(mockKiosk.saveAs).toHaveBeenCalled();
    expect(fileWriter.write).toHaveBeenCalledWith(screenshotBuffer);
  });
});
