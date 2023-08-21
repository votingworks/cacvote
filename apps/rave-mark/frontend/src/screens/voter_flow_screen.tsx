import { assert, find, throwIllegalValue } from '@votingworks/basics';
import {
  ContestId,
  OptionalVote,
  VotesDict,
  getContests,
} from '@votingworks/types';
import { useState } from 'react';
import { getElectionConfiguration, getVoterStatus } from '../api';
import * as Registration from './registration';
import * as Voting from './voting';

interface InitState {
  type: 'init';
}

interface MarkState {
  type: 'mark';
  contestIndex: number;
  votes: VotesDict;
}

interface ReviewState {
  type: 'review';
  votes: VotesDict;
  contestIndex?: number;
}

interface PrintState {
  type: 'print';
  votes: VotesDict;
}

type VoterFlowState = InitState | MarkState | ReviewState | PrintState;

function RegisteredStateScreen(): JSX.Element | null {
  const getElectionConfigurationQuery = getElectionConfiguration.useQuery();
  const electionConfiguration = getElectionConfigurationQuery.data;
  const [voterFlowState, setVoterFlowState] = useState<VoterFlowState>({
    type: 'init',
  });

  if (!electionConfiguration) {
    return null;
  }

  const { electionDefinition, ballotStyleId, precinctId } =
    electionConfiguration;
  const ballotStyle = find(
    electionDefinition.election.ballotStyles,
    (bs) => bs.id === ballotStyleId
  );
  const contests = getContests({
    election: electionDefinition.election,
    ballotStyle,
  });

  function onStartVoting() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'init');
      return {
        type: 'mark',
        contestIndex: 0,
        votes: {},
      };
    });
  }

  function onReviewContestAtIndex(contestIndex: number) {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review');
      return {
        ...prev,
        contestIndex,
      };
    });
  }

  function onReviewConfirm() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review');
      return {
        type: 'print',
        votes: prev.votes,
      };
    });
  }

  function updateVote(contestId: ContestId, vote: OptionalVote) {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'mark' || prev?.type === 'review');
      return {
        ...prev,
        votes: {
          ...prev.votes,
          [contestId]: vote,
        },
      };
    });
  }

  function goMarkNext() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'mark');
      if (prev.contestIndex === contests.length - 1) {
        return {
          type: 'review',
          votes: prev.votes,
        };
      }

      return {
        ...prev,
        contestIndex: prev.contestIndex + 1,
      };
    });
  }

  function onReturnToReview() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review');
      return {
        type: 'review',
        votes: prev.votes,
      };
    });
  }

  function goMarkPrevious() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'mark');
      return {
        ...prev,
        contestIndex: Math.max(0, prev.contestIndex - 1),
      };
    });
  }

  switch (voterFlowState.type) {
    case 'init':
      return (
        <Voting.StartScreen
          electionDefinition={electionDefinition}
          onStartVoting={onStartVoting}
        />
      );

    case 'mark':
      return (
        <Voting.MarkScreen
          electionDefinition={electionDefinition}
          contests={contests}
          contestIndex={voterFlowState.contestIndex}
          votes={voterFlowState.votes}
          updateVote={updateVote}
          goNext={goMarkNext}
          goPrevious={goMarkPrevious}
        />
      );

    case 'review':
      if (typeof voterFlowState.contestIndex === 'number') {
        return (
          <Voting.ReviewMarkScreen
            electionDefinition={electionDefinition}
            contests={contests}
            contestIndex={voterFlowState.contestIndex}
            votes={voterFlowState.votes}
            updateVote={updateVote}
            onReturnToReview={onReturnToReview}
          />
        );
      }

      return (
        <Voting.ReviewScreen
          electionDefinition={electionDefinition}
          contests={contests}
          precinctId={precinctId}
          votes={voterFlowState.votes}
          goToIndex={onReviewContestAtIndex}
          onConfirm={onReviewConfirm}
        />
      );

    case 'print':
      return (
        <Voting.PrintScreen
          electionDefinition={electionDefinition}
          ballotStyleId={ballotStyleId}
          precinctId={precinctId}
          votes={voterFlowState.votes}
          generateBallotId={() => ''}
          // TODO: use live vs test mode?
          isLiveMode={false}
        />
      );

    default:
      throwIllegalValue(voterFlowState);
  }
}

export function VoterFlowScreen(): JSX.Element | null {
  const getVoterStatusQuery = getVoterStatus.useQuery();
  const voterStatus = getVoterStatusQuery.data;

  if (!voterStatus) {
    return null;
  }

  switch (voterStatus.status) {
    case 'unregistered':
      return <Registration.StartScreen />;

    case 'registration_pending':
      return <Registration.StatusScreen />;

    case 'registered': {
      return <RegisteredStateScreen />;
    }

    case 'voted':
      return <Voting.DoneScreen />;

    default:
      throwIllegalValue(voterStatus.status);
  }
}
