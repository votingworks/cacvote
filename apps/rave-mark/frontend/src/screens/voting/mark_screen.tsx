import { assert, throwIllegalValue } from '@votingworks/basics';
import { Contest as MarkFlowContest } from '@votingworks/mark-flow-ui';
import {
  ContestId,
  Contests,
  ElectionDefinition,
  OptionalVote,
  VotesDict,
} from '@votingworks/types';
import { Button, DisplaySettings, Modal, Screen } from '@votingworks/ui';
import { useState } from 'react';
import { ButtonFooter } from '../../components/button_footer';
import { DisplaySettingsButton } from '../../components/display_settings_button';

export interface MarkScreenProps {
  electionDefinition: ElectionDefinition;
  contests: Contests;
  contestIndex: number;
  votes: VotesDict;
  updateVote: (contestId: ContestId, vote: OptionalVote) => void;
  goNext?: () => void;
  goPrevious?: () => void;
}

function noop(): void {
  // Do nothing
}

export function MarkScreen({
  electionDefinition,
  contests,
  contestIndex,
  votes,
  updateVote,
  goNext,
  goPrevious,
}: MarkScreenProps): JSX.Element | null {
  assert(contestIndex >= 0 && contestIndex < contests.length);

  const contest = contests[contestIndex];
  const hasFinishedVotingInThisContest =
    contest.type === 'candidate'
      ? (votes[contest.id]?.length ?? 0) === contest.seats
      : contest.type === 'yesno'
      ? votes[contest.id] !== undefined
      : throwIllegalValue(contest);
  const [isShowingDisplaySettings, setIsShowingDisplaySettings] =
    useState(false);

  function onPressDisplaySettingsButton() {
    setIsShowingDisplaySettings(true);
  }

  function onCloseDisplaySettings() {
    setIsShowingDisplaySettings(false);
  }

  return (
    <Screen>
      {isShowingDisplaySettings && (
        <Modal
          title="Display Settings"
          content={<DisplaySettings onClose={onCloseDisplaySettings} />}
        />
      )}
      <MarkFlowContest
        election={electionDefinition.election}
        contest={contest}
        votes={votes}
        updateVote={updateVote}
      />
      <ButtonFooter>
        <DisplaySettingsButton onPress={onPressDisplaySettingsButton} />
        <Button
          onPress={goPrevious ?? noop}
          disabled={!goPrevious}
          variant="previous"
        >
          Previous
        </Button>
        <Button
          onPress={goNext ?? noop}
          disabled={!goNext}
          variant={hasFinishedVotingInThisContest ? 'next' : 'nextSecondary'}
        >
          Next
        </Button>
      </ButtonFooter>
    </Screen>
  );
}
