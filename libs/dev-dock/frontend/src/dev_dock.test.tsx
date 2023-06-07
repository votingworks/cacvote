import { render, screen, waitFor } from '@testing-library/react';
import { Buffer } from 'buffer';
import userEvent from '@testing-library/user-event';
import { createMockClient, MockClient } from '@votingworks/grout-test-utils';
import type { Api } from '@votingworks/dev-dock-backend';
import {
  BooleanEnvironmentVariableName,
  getFeatureFlagMock,
} from '@votingworks/utils';
import {
  fakeKiosk,
  fakeFileWriter,
  fakeRaveVoterUser,
} from '@votingworks/test-utils';
import { CardStatus } from '@votingworks/auth';
import { DevDock } from './dev_dock';

const noCardStatus: CardStatus = {
  status: 'no_card',
};
const raveUnregisteredVoterCardStatus: CardStatus = {
  status: 'ready',
  cardDetails: {
    user: fakeRaveVoterUser({
      commonAccessCardId: 'voter-unregistered',
    }),
  },
};
const raveRegisteredAdminCardStatus: CardStatus = {
  status: 'ready',
  cardDetails: {
    user: fakeRaveVoterUser({
      commonAccessCardId: 'admin-registered',
    }),
  },
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
  mockApiClient.getElection.expectCallWith().resolves({
    title: 'Sample General Election',
    path: 'libs/fixtures/data/electionSample.json',
  });
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
  mockApiClient.getElection.reset();
  mockApiClient.getUsbDriveStatus.reset();
  featureFlagMock.disableFeatureFlag(
    BooleanEnvironmentVariableName.ENABLE_DEV_DOCK
  );
  const { container } = render(<DevDock apiClient={mockApiClient} />);
  expect(container).toBeEmptyDOMElement();
});

test('card mock controls', async () => {
  render(<DevDock apiClient={mockApiClient} />);

  // Card controls should enable once status loads
  const unregisteredVoterControl = screen.getByRole('button', {
    name: 'Unreg.',
  });
  await waitFor(() => {
    expect(unregisteredVoterControl).toBeEnabled();
  });
  const registeredAdminControl = screen.getByRole('button', {
    name: 'Reg. (Admin)',
  });
  expect(registeredAdminControl).toBeEnabled();

  // Insert unregistered voter card
  mockApiClient.insertCard
    .expectCallWith({
      role: 'rave_voter',
      commonAccessCardId: 'voter-unregistered',
    })
    .resolves();
  mockApiClient.getCardStatus
    .expectCallWith()
    .resolves(raveUnregisteredVoterCardStatus);
  userEvent.click(unregisteredVoterControl);
  await waitFor(() => {
    expect(registeredAdminControl).toBeDisabled();
  });

  // Remove unregistered voter card
  mockApiClient.removeCard.expectCallWith().resolves();
  mockApiClient.getCardStatus.expectCallWith().resolves(noCardStatus);
  userEvent.click(unregisteredVoterControl);
  await waitFor(() => {
    expect(registeredAdminControl).toBeEnabled();
  });

  // Insert registered admin card
  mockApiClient.insertCard
    .expectCallWith({
      role: 'rave_voter',
      commonAccessCardId: 'admin-registered',
    })
    .resolves();
  mockApiClient.getCardStatus
    .expectCallWith()
    .resolves(raveRegisteredAdminCardStatus);
  userEvent.click(registeredAdminControl);
  await waitFor(() => {
    expect(unregisteredVoterControl).toBeDisabled();
  });

  // Remove registered admin card
  mockApiClient.removeCard.expectCallWith().resolves();
  mockApiClient.getCardStatus.expectCallWith().resolves(noCardStatus);
  userEvent.click(registeredAdminControl);
  await waitFor(() => {
    expect(unregisteredVoterControl).toBeEnabled();
  });
});

test('disabled card mock controls if card mocks are disabled', async () => {
  featureFlagMock.disableFeatureFlag(
    BooleanEnvironmentVariableName.USE_MOCK_CARDS
  );
  render(<DevDock apiClient={mockApiClient} />);

  screen.getByText('Smart card mocks disabled');
  const registeredAdminControl = screen.getByRole('button', {
    name: 'Reg. (Admin)',
  });
  const unregisteredVoterControl = screen.getByRole('button', {
    name: 'Unreg.',
  });
  // Since the controls are disabled until the card status loads, we need to
  // wait for the API call to complete before checking that the controls are
  // still disabled.
  await waitFor(() => mockApiClient.assertComplete());
  expect(registeredAdminControl).toBeDisabled();
  expect(unregisteredVoterControl).toBeDisabled();
});

test('election selector', async () => {
  render(<DevDock apiClient={mockApiClient} />);
  const electionSelector = screen.getByRole('combobox');
  await waitFor(() => {
    expect(electionSelector).toHaveValue(
      'libs/fixtures/data/electionSample.json'
    );
  });

  mockApiClient.setElection
    .expectCallWith({
      path: 'libs/fixtures/data/electionFamousNames2021/election.json',
    })
    .resolves();
  mockApiClient.getElection.expectCallWith().resolves({
    title: 'Famous Names',
    path: 'libs/fixtures/data/electionFamousNames2021/election.json',
  });
  userEvent.selectOptions(
    electionSelector,
    screen.getByRole('option', { name: /Famous Names/ })
  );
  await waitFor(() => {
    expect(electionSelector).toHaveValue(
      'libs/fixtures/data/electionFamousNames2021/election.json'
    );
  });
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
