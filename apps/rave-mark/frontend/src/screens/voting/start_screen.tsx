import { Button, H1, Main, P, Screen } from '@votingworks/ui';
import { getElectionDefinition } from '../../api';

export interface StartScreenProps {
  onStartVoting: () => void;
}

export function StartScreen({
  onStartVoting,
}: StartScreenProps): JSX.Element | null {
  const getElectionDefinitionQuery = getElectionDefinition.useQuery();
  const electionDefinition = getElectionDefinitionQuery.data;

  if (!electionDefinition) {
    return null;
  }

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
