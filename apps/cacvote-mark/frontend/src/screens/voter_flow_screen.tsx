import { assert, find, throwIllegalValue } from '@votingworks/basics';
import {
  ContestId,
  OptionalVote,
  VotesDict,
  getContests,
} from '@votingworks/types';
import { useEffect, useState } from 'react';
import { Uuid, VoterStatus } from '@votingworks/cacvote-mark-backend';
import { getElectionConfiguration, getVoterStatus } from '../api';
import * as Registration from './registration';
import * as Voting from './voting';
import { randomInt } from '../random';

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
  serialNumber: number;
}

interface ReviewPrintedBallotState {
  type: 'review_printed';
  votes: VotesDict;
  serialNumber: number;
}

interface DestroyPrintedBallotState {
  type: 'destroy_printed';
  votes: VotesDict;
  serialNumber: number;
}

interface SubmitState {
  type: 'submit';
  votes: VotesDict;
  serialNumber: number;
}

interface PostVoteState {
  type: 'post_vote';
  castBallotObjectId: Uuid;
}

interface PromptToRemoveCommonAccessCardState {
  type: 'prompt_to_remove_common_access_card';
  castBallotObjectId: Uuid;
}

interface PrintMailingLabelState {
  type: 'print_mailing_label';
  castBallotObjectId: Uuid;
}

interface AttachMailingLabelState {
  type: 'attach_mailing_label';
  castBallotObjectId: Uuid;
}

interface MailBallotState {
  type: 'mail_ballot';
  castBallotObjectId: Uuid;
}

interface FinishState {
  type: 'finish';
}

type VoterFlowState =
  | InitState
  | MarkState
  | ReviewOnscreenState
  | PrintBallotState
  | ReviewPrintedBallotState
  | DestroyPrintedBallotState
  | SubmitState
  | PostVoteState
  | PromptToRemoveCommonAccessCardState
  | PrintMailingLabelState
  | AttachMailingLabelState
  | MailBallotState
  | FinishState;

interface RegisteredStateScreenProps {
  voterStatus?: VoterStatus;
  setIsVoterSessionStillActive: (isVotingSessionStillActive: boolean) => void;
}

function RegisteredStateScreen({
  voterStatus,
  setIsVoterSessionStillActive,
}: RegisteredStateScreenProps): JSX.Element | null {
  const getElectionConfigurationQuery = getElectionConfiguration.useQuery();
  const electionConfiguration = getElectionConfigurationQuery.data;
  const [voterFlowState, setVoterFlowState] = useState<VoterFlowState>({
    type: 'init',
  });

  useEffect(() => {
    if (
      voterFlowState.type === 'prompt_to_remove_common_access_card' &&
      !voterStatus
    ) {
      // the voter has removed the common access card, so we can proceed
      setVoterFlowState((prev) => {
        assert(prev?.type === 'prompt_to_remove_common_access_card');
        return {
          type: 'print_mailing_label',
          castBallotObjectId: prev.castBallotObjectId,
        };
      });
    }
  }, [voterFlowState, voterStatus]);

  if (voterStatus && !electionConfiguration) {
    return null;
  }

  const ballotStyle =
    electionConfiguration &&
    find(
      electionConfiguration.electionDefinition.election.ballotStyles,
      (bs) => bs.id === electionConfiguration.ballotStyleId
    );
  const contests =
    electionConfiguration && ballotStyle
      ? getContests({
          election: electionConfiguration.electionDefinition.election,
          ballotStyle,
        })
      : [];

  function onStartVoting() {
    setIsVoterSessionStillActive(true);
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
      const serialNumber = randomInt();
      return {
        type: 'print_ballot',
        votes: prev.votes,
        serialNumber,
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
        serialNumber: prev.serialNumber,
      };
    });
  }

  function onConfirmPrintedBallotSelections() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review_printed');
      return {
        type: 'submit',
        votes: prev.votes,
        serialNumber: prev.serialNumber,
      };
    });
  }

  function onRejectPrintedBallotSelections() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'review_printed');
      return {
        type: 'destroy_printed',
        votes: prev.votes,
        serialNumber: prev.serialNumber,
      };
    });
  }

  function onConfirmBallotDestroyed() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'destroy_printed');
      return {
        type: 'review_onscreen',
        votes: prev.votes,
      };
    });
  }

  function onCancelBallotDestroyed() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'destroy_printed');
      return {
        type: 'review_printed',
        votes: prev.votes,
        serialNumber: prev.serialNumber,
      };
    });
  }

  function onSubmitSuccess(castBallotObjectId: Uuid) {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'submit');
      return {
        type: 'post_vote',
        castBallotObjectId,
      };
    });
  }

  function onCancelSubmit() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'submit');
      return {
        type: 'review_printed',
        votes: prev.votes,
        serialNumber: prev.serialNumber,
      };
    });
  }

  function onConfirmBallotSealedInEnvelope() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'post_vote');
      return {
        type: 'prompt_to_remove_common_access_card',
        castBallotObjectId: prev.castBallotObjectId,
      };
    });
  }

  function onReturnToSealBallotInEnvelope() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'prompt_to_remove_common_access_card');
      return {
        type: 'post_vote',
        castBallotObjectId: prev.castBallotObjectId,
      };
    });
  }

  function onMailingLabelPrintCompleted() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'print_mailing_label');
      return {
        type: 'attach_mailing_label',
        castBallotObjectId: prev.castBallotObjectId,
      };
    });
  }

  function onConfirmMailingLabelAttached() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'attach_mailing_label');
      return {
        type: 'mail_ballot',
        castBallotObjectId: prev.castBallotObjectId,
      };
    });
  }

  function onReprintMailingLabelPressed() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'attach_mailing_label');
      return {
        type: 'print_mailing_label',
        castBallotObjectId: prev.castBallotObjectId,
      };
    });
  }

  function onReturnToAttachMailingLabel() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'mail_ballot');
      return {
        type: 'attach_mailing_label',
        castBallotObjectId: prev.castBallotObjectId,
      };
    });
  }

  function onConfirmBallotMailInstructions() {
    setVoterFlowState((prev) => {
      assert(prev?.type === 'mail_ballot');
      return {
        type: 'finish',
      };
    });
  }

  function onFinishedScreenDismissed() {
    setIsVoterSessionStillActive(false);
    setVoterFlowState({ type: 'init' });
  }

  switch (voterFlowState.type) {
    case 'init':
      return electionConfiguration ? (
        <Voting.StartScreen
          electionDefinition={electionConfiguration.electionDefinition}
          onStartVoting={onStartVoting}
        />
      ) : null;

    case 'mark':
      return electionConfiguration ? (
        <Voting.MarkScreen
          electionDefinition={electionConfiguration.electionDefinition}
          contests={contests}
          contestIndex={voterFlowState.contestIndex}
          votes={voterFlowState.votes}
          updateVote={updateVote}
          goNext={goMarkNext}
          goPrevious={goMarkPrevious}
        />
      ) : null;

    case 'review_onscreen':
      if (typeof voterFlowState.contestIndex === 'number') {
        return electionConfiguration ? (
          <Voting.ReviewMarkScreen
            electionDefinition={electionConfiguration.electionDefinition}
            contests={contests}
            contestIndex={voterFlowState.contestIndex}
            votes={voterFlowState.votes}
            updateVote={updateVote}
            onReturnToReview={onReturnToReview}
          />
        ) : null;
      }

      return electionConfiguration ? (
        <Voting.ReviewOnscreenBallotScreen
          electionDefinition={electionConfiguration.electionDefinition}
          contests={contests}
          precinctId={electionConfiguration.precinctId}
          votes={voterFlowState.votes}
          goToIndex={onReviewContestAtIndex}
          onConfirm={onReviewConfirm}
        />
      ) : null;

    case 'print_ballot':
      return electionConfiguration ? (
        <Voting.PrintBallotScreen
          electionDefinition={electionConfiguration.electionDefinition}
          ballotStyleId={electionConfiguration.ballotStyleId}
          precinctId={electionConfiguration.precinctId}
          votes={voterFlowState.votes}
          generateBallotId={() => `${voterFlowState.serialNumber}`}
          // TODO: use live vs test mode?
          isLiveMode={false}
          onPrintCompleted={onPrintBallotCompleted}
        />
      ) : null;

    case 'review_printed':
      return (
        <Voting.ReviewPrintedBallotScreen
          onConfirm={onConfirmPrintedBallotSelections}
          onReject={onRejectPrintedBallotSelections}
        />
      );

    case 'destroy_printed':
      return (
        <Voting.DestroyBallotScreen
          onConfirm={onConfirmBallotDestroyed}
          onCancel={onCancelBallotDestroyed}
        />
      );

    case 'submit':
      return (
        <Voting.SubmitScreen
          votes={voterFlowState.votes}
          serialNumber={voterFlowState.serialNumber}
          onSubmitSuccess={onSubmitSuccess}
          onCancel={onCancelSubmit}
        />
      );

    case 'post_vote':
      return (
        <Voting.SealBallotInEnvelopeScreen
          onNext={onConfirmBallotSealedInEnvelope}
        />
      );

    case 'prompt_to_remove_common_access_card':
      return (
        <Voting.RemoveCommonAccessCardToPrintMailLabelScreen
          onCancel={onReturnToSealBallotInEnvelope}
        />
      );

    case 'print_mailing_label':
      return (
        <Voting.PrintMailingLabelScreen
          castBallotObjectId={voterFlowState.castBallotObjectId}
          onPrintCompleted={onMailingLabelPrintCompleted}
        />
      );

    case 'attach_mailing_label':
      return (
        <Voting.AttachMailingLabelScreen
          onConfirmMailingLabelAttached={onConfirmMailingLabelAttached}
          onReprintMailingLabelPressed={onReprintMailingLabelPressed}
        />
      );

    case 'mail_ballot':
      return (
        <Voting.MailBallotScreen
          onBack={onReturnToAttachMailingLabel}
          onDone={onConfirmBallotMailInstructions}
        />
      );

    case 'finish':
      return <Voting.FinishedScreen onDone={onFinishedScreenDismissed} />;

    default:
      throwIllegalValue(voterFlowState);
  }
}

export interface VoterFlowScreenProps {
  setIsVoterSessionStillActive: (isVoterSessionStillActive: boolean) => void;
}

export function VoterFlowScreen({
  setIsVoterSessionStillActive,
}: VoterFlowScreenProps): JSX.Element | null {
  const getVoterStatusQuery = getVoterStatus.useQuery();
  const voterStatus = getVoterStatusQuery.data?.status;

  switch (voterStatus) {
    case 'unregistered':
      return <Registration.StartScreen />;

    case 'registration_pending':
      return <Registration.StatusScreen />;

    case 'registered':
    case undefined:
      return (
        <RegisteredStateScreen
          voterStatus={voterStatus}
          setIsVoterSessionStillActive={setIsVoterSessionStillActive}
        />
      );

    case 'voted':
      return <Voting.AlreadyVotedScreen />;

    default:
      throwIllegalValue(voterStatus);
  }
}
