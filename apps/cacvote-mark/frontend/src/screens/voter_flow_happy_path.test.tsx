import { QueryClientProvider } from '@tanstack/react-query';
import { act, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Result, assertDefined, ok } from '@votingworks/basics';
import { electionFamousNames2021Fixtures } from '@votingworks/fixtures';
import { createMockClient } from '@votingworks/grout-test-utils';
import { useBallotPrinter } from '@votingworks/mark-flow-ui';
import { getBallotStyle, getContests } from '@votingworks/types';
import { renderWithThemes } from '@votingworks/ui';
import { ApiClient, ApiClientContext, createQueryClient } from '../api';
import { randomInt } from '../random';
import { VoterFlowScreen } from './voter_flow_screen';

type Uuid = ReturnType<ApiClient['castBallot']> extends Promise<
  Result<infer T extends { id: string }, unknown>
>
  ? T['id']
  : never;

jest.mock('@votingworks/mark-flow-ui', () => ({
  ...jest.requireActual('@votingworks/mark-flow-ui'),
  useBallotPrinter: jest.fn(),
}));

jest.mock('../random', () => ({
  randomInt: jest.fn(),
}));

const useBallotPrinterMock = useBallotPrinter as jest.Mock<
  ReturnType<typeof useBallotPrinter>,
  Parameters<typeof useBallotPrinter>
>;

const randomIntMock = randomInt as jest.Mock;

test('voter flow happy path', async () => {
  jest.useFakeTimers();

  let useBallotPrinterMockOptions:
    | Parameters<typeof useBallotPrinter>[0]
    | undefined;
  const printBallotMock = jest.fn();
  useBallotPrinterMock.mockImplementation((options) => {
    useBallotPrinterMockOptions = options;
    return printBallotMock;
  });

  const serialNumber = 1234567890;
  const pin = '77777777';
  const uuid = '00000000-0000-0000-0000-000000000000' as Uuid;

  randomIntMock.mockReturnValue(serialNumber);

  const apiClient = createMockClient<ApiClient>();
  const queryClient = createQueryClient();

  apiClient.getVoterStatus.expectRepeatedCallsWith().resolves({
    status: 'registered',
  });

  const { electionDefinition } = electionFamousNames2021Fixtures;
  const { election } = electionDefinition;
  const ballotStyleId = '1';
  const precinctId = '20';
  const ballotStyle = assertDefined(
    getBallotStyle({ election, ballotStyleId })
  );

  apiClient.getElectionConfiguration.expectRepeatedCallsWith().resolves({
    electionDefinition,
    ballotStyleId,
    precinctId,
  });

  apiClient.castBallot
    .expectCallWith({
      votes: {},
      serialNumber,
      pin,
    })
    .resolves(ok({ id: uuid }));

  renderWithThemes(
    <ApiClientContext.Provider value={apiClient}>
      <QueryClientProvider client={queryClient}>
        <VoterFlowScreen />
      </QueryClientProvider>
    </ApiClientContext.Provider>
  );

  // Start voting
  await screen.findByText('Ready to Vote');
  userEvent.click(screen.getByRole('button', { name: 'Start Voting' }));

  // Skip all the way to the end
  for (const contest of getContests({ election, ballotStyle })) {
    await screen.findByText(contest.title);
    userEvent.click(screen.getByRole('button', { name: 'Next' }));
  }

  // Done voting, review onscreen selections
  await screen.findByText('Review Your Votes');
  userEvent.click(screen.getByText('Print My Ballot'));

  await screen.findByText(/Printing Your Official Ballot/);
  expect(printBallotMock).toHaveBeenCalled();
  useBallotPrinterMockOptions?.onPrintStarted?.();

  // Wait for the ballot to print
  act(() => {
    jest.advanceTimersByTime(10_000);
  });

  // Review the printed ballot, follow the flow to cast it
  await screen.findByText('Review Your Ballot');
  userEvent.click(screen.getByLabelText(/Yes/));
  userEvent.click(screen.getByRole('button', { name: /Enter PIN/i }));

  for (const digit of pin) {
    userEvent.click(screen.getByRole('button', { name: digit }));
  }
  userEvent.click(screen.getByRole('button', { name: 'enter' }));

  // Wait for the ballot to be cast, then move on to print the mail label
  await screen.findByText('Step 2');

  userEvent.click(
    screen.getByRole('button', { name: 'Step 3: Print Mail Label' })
  );

  // Wait for the prompt to remove the common access card
  await screen.findByText('Step 3');

  // Try going back to the previous step
  userEvent.click(
    screen.getByRole('button', { name: 'Step 2: Seal Ballot in Envelope' })
  );

  // Ensure we're back at the previous step
  await screen.findByText('Step 2');

  userEvent.click(
    screen.getByRole('button', { name: 'Step 3: Print Mail Label' })
  );

  // Wait for the prompt to remove the common access card
  await screen.findByText('Step 3');

  apiClient.assertComplete();
});
