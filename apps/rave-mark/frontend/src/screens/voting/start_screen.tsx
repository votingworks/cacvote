import { Button, H1, Main, P, Screen } from '@votingworks/ui';
import { getElectionConfiguration } from '../../api';

export interface StartScreenProps {
  onStartVoting: () => void;
}

export function StartScreen({
  onStartVoting,
}: StartScreenProps): JSX.Element | null {
  const getElectionConfigurationQuery = getElectionConfiguration.useQuery();
  const electionConfiguration = getElectionConfigurationQuery.data;

  if (!electionConfiguration) {
    return null;
  }

  return (
    <Screen>
      <Main>
        <H1>Voting</H1>
        <P>{electionConfiguration.electionDefinition.election.title}</P>
        <Button onPress={onStartVoting}>Start Voting</Button>
      </Main>
    </Screen>
  );
}
