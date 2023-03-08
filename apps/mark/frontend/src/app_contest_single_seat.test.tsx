import React from 'react';
import { MemoryStorage, MemoryHardware } from '@votingworks/utils';

import userEvent from '@testing-library/user-event';
import { fireEvent, render, act, screen } from '../test/react_testing_library';
import { App } from './app';

import { advanceTimersAndPromises } from '../test/helpers/timers';

import {
  presidentContest,
  setElectionInStorage,
  setStateInStorage,
} from '../test/helpers/election';
import { ApiMock, createApiMock } from '../test/helpers/mock_api_client';

let apiMock: ApiMock;

beforeEach(() => {
  jest.useFakeTimers();
  window.location.href = '/';
  apiMock = createApiMock();
});

afterEach(() => {
  apiMock.mockApiClient.assertComplete();
});

it('Single Seat Contest', async () => {
  // ====================== BEGIN CONTEST SETUP ====================== //

  apiMock.expectGetMachineConfig();
  const hardware = MemoryHardware.buildStandard();
  const storage = new MemoryStorage();

  await setElectionInStorage(storage);
  await setStateInStorage(storage);

  const { container } = render(
    <App
      hardware={hardware}
      storage={storage}
      reload={jest.fn()}
      apiClient={apiMock.mockApiClient}
    />
  );
  await advanceTimersAndPromises();

  // Start voter session
  apiMock.setAuthStatusCardlessVoterLoggedIn({
    ballotStyleId: '12',
    precinctId: '23',
  });

  // Go to First Contest
  userEvent.click(await screen.findByText('Start Voting'));
  await advanceTimersAndPromises();

  // ====================== END CONTEST SETUP ====================== //

  const candidate0 = presidentContest.candidates[0].name;
  const candidate1 = presidentContest.candidates[1].name;

  screen.getByText(presidentContest.title);

  // Select first candidate
  fireEvent.click(screen.getByText(candidate0));
  await advanceTimersAndPromises();

  // Select second candidate
  fireEvent.click(screen.getByText(candidate1));
  await advanceTimersAndPromises();

  // Overvote modal is displayed
  screen.getByText(
    `You may only select ${presidentContest.seats} candidate in this contest. To vote for ${candidate1}, you must first unselect the selected candidate.`
  );

  // Capture styles of Single Candidate Contest
  expect(container.firstChild).toMatchSnapshot();

  // Close the modal
  fireEvent.click(screen.getByText('Okay'));
  await advanceTimersAndPromises();

  // First candidate is selected
  expect(
    screen.getByText(candidate0).closest('button')!.dataset['selected']
  ).toEqual('true');

  // Second candidate is NOT selected
  expect(
    screen.getByText(candidate1).closest('button')!.dataset['selected']
  ).toEqual('false');

  // Deselect the first candidate
  fireEvent.click(screen.getByText(candidate0));

  // Check that the aria label was updated to be include 'deselected' and is then updated back to the original state
  expect(
    screen.getByText(candidate0).closest('button')?.getAttribute('aria-label')
  ).toContain('Deselected,');
  expect(
    screen.getByText(candidate1).closest('button')?.getAttribute('aria-label')
  ).not.toContain('Deselected,');
  act(() => {
    jest.advanceTimersByTime(101);
  });
  expect(
    screen.getByText(candidate0).closest('button')?.getAttribute('aria-label')
  ).not.toContain('Deselected,');

  await advanceTimersAndPromises();
});