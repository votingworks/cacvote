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

interface ReviewOnscreenState {
  type: 'review_onscreen';
  votes: VotesDict;
  contestIndex?: number;
}

interface PrintBallotState {
  type: 'print_ballot';
  votes: VotesDict;
}

interface PrintMailingLabelState {
  type: 'print_mailing_label';
  votes: VotesDict;
}

interface ReviewPrintedBallotState {
  type: 'review_printed';
  votes: VotesDict;
}

interface SubmitState {
  type: 'submit';
  votes: VotesDict;
}

interface PostVoteState {
  type: 'post_vote';
}

type VoterFlowState =
  | InitState
  | MarkState
  | ReviewOnscreenState
  | PrintBallotState
  | ReviewPrintedBallotState
  | SubmitState
  | PrintMailingLabelState
  | PostVoteState;

interface RegisteredStateScreenProps {
  onIsVotingSessionInProgressChanged: (
    isVotingSessionInProgress: boolean
  ) => void;
}

function RegisteredStateScreen({
  onIsVotingSessionInProgressChanged,
}: RegisteredStateScreenProps): JSX.Element | null {
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
    onIsVotingSessionInProgressChanged(true);
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
      assert(prev?.type === 'review_onscreen');
      return {
        ...prev,
        contestIndex,
      };
    });
  }

  function onReviewConfirm() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review_onscreen');
      return {
        type: 'print_ballot',
        votes: prev.votes,
      };
    });
  }

  function updateVote(contestId: ContestId, vote: OptionalVote) {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'mark' || prev?.type === 'review_onscreen');
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
          type: 'review_onscreen',
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
      assert(prev?.type === 'review_onscreen');
      return {
        type: 'review_onscreen',
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

  function onPrintBallotCompleted() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'print_ballot');
      return {
        type: 'review_printed',
        votes: prev.votes,
      };
    });
  }

  function onConfirmPrintedBallotSelections() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review_printed');
      return {
        type: 'submit',
        votes: prev.votes,
      };
    });
  }

  function onRejectPrintedBallotSelections() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review_printed');
      return {
        type: 'review_onscreen',
        votes: prev.votes,
      };
    });
  }

  function onSubmitted() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'submit');
      return {
        type: 'print_mailing_label',
        votes: prev.votes,
      };
    });
  }

  function onPrintMailingLabelCompleted() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'print_mailing_label');
      return {
        type: 'post_vote',
        votes: prev.votes,
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

    case 'review_onscreen':
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
        <Voting.ReviewOnscreenBallotScreen
          electionDefinition={electionDefinition}
          contests={contests}
          precinctId={precinctId}
          votes={voterFlowState.votes}
          goToIndex={onReviewContestAtIndex}
          onConfirm={onReviewConfirm}
        />
      );

    case 'print_ballot':
      return (
        <Voting.PrintBallotScreen
          electionDefinition={electionDefinition}
          ballotStyleId={ballotStyleId}
          precinctId={precinctId}
          votes={voterFlowState.votes}
          generateBallotId={() => ''}
          // TODO: use live vs test mode?
          isLiveMode={false}
          onPrintCompleted={onPrintBallotCompleted}
        />
      );

    case 'review_printed':
      return (
        <Voting.ReviewPrintedBallotScreen
          onConfirm={onConfirmPrintedBallotSelections}
          onReject={onRejectPrintedBallotSelections}
        />
      );

    case 'submit':
      return (
        <Voting.SubmitScreen
          votes={voterFlowState.votes}
          onSubmitted={onSubmitted}
        />
      );

    case 'print_mailing_label':
      return (
        <Voting.PrintMailingLabelScreen
          onPrintCompleted={onPrintMailingLabelCompleted}
        />
      );

    case 'post_vote':
      return <Voting.PostVoteScreen />;

    default:
      throwIllegalValue(voterFlowState);
  }
}

export function VoterFlowScreen(): JSX.Element | null {
  const [isVotingSessionInProgress, setIsVotingSessionInProgress] =
    useState(false);
  const getVoterStatusQuery = getVoterStatus.useQuery();
  const voterStatus = getVoterStatusQuery.data;

  if (!voterStatus) {
    return null;
  }

  if (isVotingSessionInProgress) {
    return (
      <RegisteredStateScreen
        onIsVotingSessionInProgressChanged={setIsVotingSessionInProgress}
      />
    );
  }

  switch (voterStatus.status) {
    case 'unregistered':
      return <Registration.StartScreen />;

    case 'registration_pending':
      return <Registration.StatusScreen />;

    case 'registered':
      return (
        <RegisteredStateScreen
          onIsVotingSessionInProgressChanged={setIsVotingSessionInProgress}
        />
      );

    case 'voted':
      return <Voting.AlreadyVotedScreen />;

    default:
      throwIllegalValue(voterStatus.status);
  }
}
