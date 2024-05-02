import {
  electionFamousNames2021Fixtures,
  electionGridLayoutNewHampshireTestBallotFixtures,
} from '@votingworks/fixtures';
import { renderWithThemes } from '@votingworks/ui';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Optional } from '@votingworks/basics';
import {
  ElectionConfiguration,
  ElectionConfigurationSelect,
} from './election_configuration_select';
import { Uuid } from '../cacvote-server/types';

test('no elections', () => {
  const onChange = jest.fn();
  renderWithThemes(
    <ElectionConfigurationSelect elections={[]} onChange={onChange} />
  );

  // still has the prompt
  screen.getByText(/Select an election configuration/i);
});

test('single election', () => {
  const onChange = jest.fn<void, [Optional<ElectionConfiguration>], void>();
  const electionId = Uuid();
  renderWithThemes(
    <ElectionConfigurationSelect
      elections={[
        {
          id: electionId,
          election: electionFamousNames2021Fixtures.election,
        },
      ]}
      onChange={onChange}
    />
  );

  userEvent.click(screen.getByText(/Select an election configuration/i));
  userEvent.click(screen.getByText(/BS: 1 - P: 21/i));

  expect(onChange).toHaveBeenCalledWith({
    electionId,
    ballotStyleId: '1',
    precinctId: '21',
  });
});

test('multiple elections', () => {
  const onChange = jest.fn<void, [Optional<ElectionConfiguration>], void>();
  const electionId1 = Uuid();
  const electionId2 = Uuid();
  renderWithThemes(
    <ElectionConfigurationSelect
      elections={[
        {
          id: electionId1,
          election: electionFamousNames2021Fixtures.election,
        },
        {
          id: electionId2,
          election: electionGridLayoutNewHampshireTestBallotFixtures.election,
        },
      ]}
      onChange={onChange}
    />
  );

  userEvent.click(screen.getByText(/Select an election configuration/i));
  userEvent.click(screen.getByText(/BS: 1 - P: 21/i));

  expect(onChange).toHaveBeenCalledWith({
    electionId: electionId1,
    ballotStyleId: '1',
    precinctId: '21',
  });

  userEvent.click(screen.getByText(/Select an election configuration/i));
  userEvent.click(
    screen.getByText(/BS: card-number-3 - P: town-id-00701-precinct-id-/i)
  );

  expect(onChange).toHaveBeenCalledWith({
    electionId: electionId2,
    ballotStyleId: 'card-number-3',
    precinctId: 'town-id-00701-precinct-id-',
  });
});

test('initial value', () => {
  const onChange = jest.fn<void, [Optional<ElectionConfiguration>], void>();
  const electionId = Uuid();
  renderWithThemes(
    <ElectionConfigurationSelect
      elections={[
        {
          id: electionId,
          election: electionFamousNames2021Fixtures.election,
        },
      ]}
      value={{
        electionId,
        ballotStyleId: '1',
        precinctId: '21',
      }}
      onChange={onChange}
    />
  );

  screen.getByText(/BS: 1 - P: 21/i);
});
