import { ElectionDefinition } from '@votingworks/types';
import { Button, H1, Main, Screen, Text } from '@votingworks/ui';
import { formatShortDate } from '@votingworks/utils';
import { DateTime } from 'luxon';

export interface StartScreenProps {
  electionDefinition: ElectionDefinition;
  onStartVoting: () => void;
}

export function StartScreen({
  electionDefinition,
  onStartVoting,
}: StartScreenProps): JSX.Element | null {
  return (
    <Screen>
      <Main centerChild>
        <H1>Ready to Vote</H1>
        <Text>{electionDefinition.election.title}</Text>
        <Text>
          {formatShortDate(DateTime.fromISO(electionDefinition.election.date))}
        </Text>
        <Button variant="primary" onPress={onStartVoting}>
          Start Voting
        </Button>
      </Main>
    </Screen>
  );
}
