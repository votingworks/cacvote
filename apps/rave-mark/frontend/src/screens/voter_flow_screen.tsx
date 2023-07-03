import { throwIllegalValue } from '@votingworks/basics';
import { ContestId, OptionalVote, VotesDict } from '@votingworks/types';
import { useState } from 'react';
import { getVoterStatus } from '../api';
import * as Registration from './registration';
import * as Voting from './voting';

interface VoterFlowState {
  contestIndex: number;
  votes: VotesDict;
}

export function VoterFlowScreen(): JSX.Element | null {
  const getVoterStatusQuery = getVoterStatus.useQuery();
  const voterStatus = getVoterStatusQuery.data;
  const [voterFlowState, setVoterFlowState] = useState<VoterFlowState>({
    contestIndex: -1,
    votes: {},
  });

  if (!voterStatus) {
    return null;
  }

  function onStartVoting() {
    setVoterFlowState((prev) => {
      return {
        contestIndex: 0,
        votes: prev?.votes ?? {},
      };
    });
  }

  function goPrevious() {
    setVoterFlowState((prev) => {
      if (!prev) {
        return prev;
      }

      return {
        ...prev,
        contestIndex: prev.contestIndex - 1,
      };
    });
  }

  function goNext() {
    setVoterFlowState((prev) => {
      if (!prev) {
        return prev;
      }

      return {
        ...prev,
        contestIndex: prev.contestIndex + 1,
      };
    });
  }

  function goToIndex(contestIndex: number) {
    setVoterFlowState((prev) => {
      if (!prev) {
        return prev;
      }

      return {
        ...prev,
        contestIndex,
      };
    });
  }

  function updateVote(contestId: ContestId, vote: OptionalVote) {
    setVoterFlowState((prev = { contestIndex: 0, votes: {} }) => ({
      ...prev,
      votes: {
        ...prev.votes,
        [contestId]: vote,
      },
    }));
  }

  switch (voterStatus.status) {
    case 'unregistered':
      return <Registration.StartScreen />;

    case 'registration_pending':
      return <Registration.StatusScreen />;

    case 'registered': {
      if (voterFlowState.contestIndex < 0) {
        return <Voting.StartScreen onStartVoting={onStartVoting} />;
      }

      return (
        <Voting.MarkScreen
          contestIndex={voterFlowState.contestIndex}
          votes={voterFlowState.votes}
          updateVote={updateVote}
          goPrevious={goPrevious}
          goNext={goNext}
          goToIndex={goToIndex}
        />
      );
    }

    case 'voted':
      return <Voting.DoneScreen />;

    default:
      throwIllegalValue(voterStatus.status);
  }
}
