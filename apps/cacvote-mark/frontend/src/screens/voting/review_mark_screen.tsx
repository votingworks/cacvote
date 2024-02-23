import { assert } from '@votingworks/basics';
import { Contest as MarkFlowContest } from '@votingworks/mark-flow-ui';
import {
  ContestId,
  Contests,
  ElectionDefinition,
  OptionalVote,
  VotesDict,
} from '@votingworks/types';
import { Button, Screen } from '@votingworks/ui';
import { ButtonFooter } from '../../components/button_footer';

export interface ReviewMarkScreenProps {
  electionDefinition: ElectionDefinition;
  contests: Contests;
  contestIndex: number;
  votes: VotesDict;
  updateVote: (contestId: ContestId, vote: OptionalVote) => void;
  onReturnToReview: () => void;
}

export function ReviewMarkScreen({
  electionDefinition,
  contests,
  contestIndex,
  votes,
  updateVote,
  onReturnToReview,
}: ReviewMarkScreenProps): JSX.Element | null {
  assert(contestIndex >= 0 && contestIndex < contests.length);
  const contest = contests[contestIndex];

  return (
    <Screen>
      <MarkFlowContest
        election={electionDefinition.election}
        contest={contest}
        votes={votes}
        updateVote={updateVote}
      />
      <ButtonFooter>
        <Button onPress={onReturnToReview} icon="Next">
          Return to Review
        </Button>
      </ButtonFooter>
    </Screen>
  );
}
