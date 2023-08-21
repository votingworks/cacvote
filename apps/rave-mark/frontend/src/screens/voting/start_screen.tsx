import { Button, H1, Main, P, Screen } from '@votingworks/ui';
import { ElectionDefinition } from '@votingworks/types';

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
      <Main>
        <H1>Voting</H1>
        <P>{electionDefinition.election.title}</P>
        <Button onPress={onStartVoting}>Start Voting</Button>
      </Main>
    </Screen>
  );
}
