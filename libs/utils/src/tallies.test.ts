import {
  electionFamousNames2021Fixtures,
  electionMinimalExhaustiveSample,
  electionSample,
} from '@votingworks/fixtures';
import {
  ContestTally,
  FullElectionTally,
  PartyIdSchema,
  unsafeParse,
} from '@votingworks/types';
import { assert } from '@votingworks/basics';
import { getEmptyTally, tallyVotesByContest } from './votes';
import {
  combineContestTallies,
  getSubTalliesByPartyAndPrecinct,
  getTallyIdentifier,
} from './tallies';
import {
  ALL_PRECINCTS_SELECTION,
  singlePrecinctSelectionFor,
} from './precinct_selection';

describe('getTallyIdentifier', () => {
  const party1 = unsafeParse(PartyIdSchema, 'party1');

  test('returns expected identifier with a party and precinct', () => {
    expect(getTallyIdentifier(party1, 'precinct1')).toEqual('party1,precinct1');
  });

  test('returns expected identifier with no party and a precinct', () => {
    expect(getTallyIdentifier(undefined, 'precinct1')).toEqual(
      'undefined,precinct1'
    );
  });

  test('returns expected identifier with a party and no precinct', () => {
    expect(getTallyIdentifier(party1)).toEqual('party1,__ALL_PRECINCTS');
  });

  test('returns expected identifier with no party and no precinct', () => {
    expect(getTallyIdentifier()).toEqual('undefined,__ALL_PRECINCTS');
  });
});

test('combineContestTallies adds tallies together', () => {
  const contestZeroTallies = tallyVotesByContest({
    election: electionSample,
    votes: [],
  });
  for (const contest of electionSample.contests) {
    const contestZeroTally = contestZeroTallies[contest.id];
    assert(contestZeroTally);
    // 0 + 0 = 0
    expect(combineContestTallies(contestZeroTally, contestZeroTally)).toEqual(
      contestZeroTally
    );
    const contestUndefinedTally: ContestTally = {
      contest: contestZeroTally.contest,
      metadata: contestZeroTally.metadata,
      tallies: {},
    };
    // 0 + undefined = 0
    expect(
      combineContestTallies(contestZeroTally, contestUndefinedTally)
    ).toEqual(contestZeroTally);
  }
});

const emptyTally: FullElectionTally = {
  overallTally: getEmptyTally(),
  resultsByCategory: new Map(),
};
describe('getSubTalliesByPartyAndPrecinct', () => {
  test('primary election and no precinct specific data', () => {
    const subTallies = getSubTalliesByPartyAndPrecinct({
      election: electionMinimalExhaustiveSample,
      tally: emptyTally,
    });

    expect(Array.from(subTallies.keys()).sort()).toEqual(
      [
        '0,__ALL_PRECINCTS',
        '1,__ALL_PRECINCTS',
        'undefined,__ALL_PRECINCTS',
      ].sort()
    );
  });

  test('general election and all precincts specific data', () => {
    const subTallies = getSubTalliesByPartyAndPrecinct({
      election: electionFamousNames2021Fixtures.election,
      tally: emptyTally,
      precinctSelection: ALL_PRECINCTS_SELECTION,
    });

    expect(Array.from(subTallies.keys()).sort()).toMatchObject(
      ['undefined,20', 'undefined,21', 'undefined,22', 'undefined,23'].sort()
    );
  });

  test('general election and single precinct data', () => {
    const subTallies = getSubTalliesByPartyAndPrecinct({
      election: electionFamousNames2021Fixtures.election,
      tally: emptyTally,
      precinctSelection: singlePrecinctSelectionFor('20'),
    });

    expect(Array.from(subTallies.keys())).toMatchObject(['undefined,20']);
  });
});
