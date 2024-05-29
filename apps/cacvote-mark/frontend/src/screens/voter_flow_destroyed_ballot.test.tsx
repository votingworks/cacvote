import { QueryClientProvider } from '@tanstack/react-query';
import { act, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { assertDefined } from '@votingworks/basics';
import { electionFamousNames2021Fixtures } from '@votingworks/fixtures';
import { createMockClient } from '@votingworks/grout-test-utils';
import { useBallotPrinter } from '@votingworks/mark-flow-ui';
import { getBallotStyle, getContests } from '@votingworks/types';
import { renderWithThemes } from '@votingworks/ui';
import { ApiClient, ApiClientContext, createQueryClient } from '../api';
import { VoterFlowScreen } from './voter_flow_screen';

jest.mock('@votingworks/mark-flow-ui', () => ({
  ...jest.requireActual('@votingworks/mark-flow-ui'),
  useBallotPrinter: jest.fn(),
}));

const useBallotPrinterMock = useBallotPrinter as jest.Mock<
  ReturnType<typeof useBallotPrinter>,
  Parameters<typeof useBallotPrinter>
>;

test('destroyed ballot', async () => {
  jest.useFakeTimers();

  let useBallotPrinterMockOptions:
    | Parameters<typeof useBallotPrinter>[0]
    | undefined;
  const printBallotMock = jest.fn();
  useBallotPrinterMock.mockImplementation((options) => {
    useBallotPrinterMockOptions = options;
    return printBallotMock;
  });

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

  renderWithThemes(
    <ApiClientContext.Provider value={apiClient}>
      <QueryClientProvider client={queryClient}>
        <VoterFlowScreen
          isVoterSessionStillActive
          setIsVoterSessionStillActive={jest.fn()}
        />
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

  // Review the printed ballot, follow the flow to destroy the ballot
  await screen.findByText('Review Your Ballot');
  userEvent.click(screen.getByLabelText(/No/));
  userEvent.click(screen.getByRole('button', { name: /Destroy Ballot/ }));

  // Cancel the destroy ballot flow
  await screen.findByText('Destroy Your Ballot');
  userEvent.click(screen.getByText('Go Back to Step 1'));

  // Back to reviewing the printed ballot, follow the flow to destroy the ballot again
  await screen.findByText('Review Your Ballot');
  userEvent.click(screen.getByLabelText(/No/));
  userEvent.click(screen.getByRole('button', { name: /Destroy Ballot/ }));

  // Actually follow through with destroying the ballot
  await screen.findByText('Destroy Your Ballot');
  userEvent.click(screen.getByText('I Destroyed My Ballot'));

  // Back to the pre-printing review screen
  await screen.findByText('Review Your Votes');

  apiClient.assertComplete();
});
