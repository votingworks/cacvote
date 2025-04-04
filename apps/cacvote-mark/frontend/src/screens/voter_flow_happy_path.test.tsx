import { Buffer } from 'buffer';
import { QueryClientProvider } from '@tanstack/react-query';
import { act, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Result, assertDefined, ok } from '@votingworks/basics';
import { electionFamousNames2021Fixtures } from '@votingworks/fixtures';
import { createMockClient } from '@votingworks/grout-test-utils';
import { getBallotStyle, getContests } from '@votingworks/types';
import { renderWithThemes, printElementToPdf } from '@votingworks/ui';
import { ApiClient, ApiClientContext, createQueryClient } from '../api';
import { randomInt } from '../random';
import { VoterFlowScreen } from './voter_flow_screen';

type Uuid = ReturnType<ApiClient['castBallot']> extends Promise<
  Result<infer T extends { id: string }, unknown>
>
  ? T['id']
  : never;

jest.mock('../random', () => ({
  randomInt: jest.fn(),
}));

jest.mock('uuid', () => ({
  v4: () => '00000000-0000-0000-0000-000000000000',
}));

const randomIntMock = randomInt as jest.Mock;

jest.mock('@votingworks/ui', () => ({
  ...jest.requireActual('@votingworks/ui'),
  printElementToPdf: jest.fn(),
}));

test('voter flow happy path', async () => {
  jest.useFakeTimers();

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
        <VoterFlowScreen
          isVoterSessionStillActive
          setIsVoterSessionStillActive={jest.fn()}
        />
      </QueryClientProvider>
    </ApiClientContext.Provider>
  );

  // Start voting
  await screen.findByRole('heading', { name: 'Ready to Vote' });
  userEvent.click(screen.getByRole('button', { name: 'Start Voting' }));

  // Skip all the way to the end
  for (const contest of getContests({ election, ballotStyle })) {
    await screen.findByText(contest.title);
    userEvent.click(screen.getByRole('button', { name: 'Next' }));
  }

  jest.mocked(printElementToPdf).mockResolvedValue(Buffer.of());
  apiClient.printBallotPdf
    .expectCallWith({ pdfData: Buffer.of() })
    .resolves(ok());

  // Done voting, review onscreen selections
  await screen.findByRole('heading', { name: 'Review Your Votes' });
  userEvent.click(screen.getByRole('button', { name: 'Print My Ballot' }));

  await screen.findByRole('heading', {
    name: /Printing Your Official Ballot/,
    level: 2,
  });
  expect(printElementToPdf).toHaveBeenCalled();

  // Wait for the ballot to print
  act(() => {
    jest.advanceTimersByTime(10_000);
  });

  // Review the printed ballot, follow the flow to cast it
  await screen.findByRole('heading', {
    name: 'Review Your Ballot',
    level: 3,
  });
  userEvent.click(screen.getByLabelText(/Yes/));
  userEvent.click(screen.getByRole('button', { name: /Enter PIN/i }));

  for (const digit of pin) {
    userEvent.click(screen.getByRole('button', { name: digit }));
  }
  userEvent.click(screen.getByRole('button', { name: 'enter' }));

  // Wait for the ballot to be cast, then move on to print the mail label
  await screen.findByRole('heading', {
    name: 'Seal Ballot in Envelope',
    level: 3,
  });

  userEvent.click(
    screen.getByRole('button', { name: 'Step 3: Print Mail Label' })
  );

  // Wait for the prompt to remove the common access card
  await screen.findByRole('heading', {
    name: 'Remove Common Access Card to Print Mail Label',
    level: 3,
  });

  // Try going back to the previous step
  userEvent.click(
    screen.getByRole('button', { name: 'Step 2: Seal Ballot in Envelope' })
  );

  // Ensure we're back at the previous step
  await screen.findByRole('heading', {
    name: 'Seal Ballot in Envelope',
    level: 3,
  });

  userEvent.click(
    screen.getByRole('button', { name: 'Step 3: Print Mail Label' })
  );

  // Wait for the prompt to remove the common access card
  await screen.findByRole('heading', {
    name: 'Remove Common Access Card to Print Mail Label',
    level: 3,
  });

  apiClient.printMailingLabel
    .expectCallWith({
      printMailLabelJobId: uuid,
      castBallotObjectId: uuid,
    })
    .resolves(ok());

  // Remove the common access card
  apiClient.getVoterStatus.expectRepeatedCallsWith().resolves(undefined);
  apiClient.getElectionConfiguration
    .expectRepeatedCallsWith()
    .resolves(undefined);
  await queryClient.invalidateQueries();

  await screen.findByRole('heading', {
    name: /Printing Mail Label/,
    level: 3,
  });

  act(() => {
    jest.runAllTimers();
  });

  // Printing is done, now attach the mail label
  await screen.findByRole('heading', {
    name: 'Attach Mail Label to Envelope',
    level: 3,
  });

  // Continue to the next step
  userEvent.click(screen.getByRole('button', { name: 'Step 4: Mail Ballot' }));

  // View instructions to mail the ballot
  await screen.findByRole('heading', { name: 'Mail Ballot', level: 3 });

  // Done with the flow
  userEvent.click(screen.getByRole('button', { name: 'Done' }));

  // Shows a reminder to mail the ballot
  await screen.findByRole('heading', {
    name: 'Don’t forget to mail your ballot.',
    level: 4,
  });

  act(() => {
    jest.runAllTimers();
  });

  apiClient.assertComplete();
});
