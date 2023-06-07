import { useState } from 'react';
import { Button, H1, Main, P, Screen } from '@votingworks/ui';
import { VoterFlowScreen } from './voter_flow_screen';

export function AdminScreen(): JSX.Element {
  const [showVoterFlow, setShowVoterFlow] = useState(false);

  if (showVoterFlow) {
    return <VoterFlowScreen />;
  }

  return (
    <Screen>
      <Main>
        <H1>Admin</H1>
        <P>Admin stuff goes here.</P>
        <P>
          <Button onPress={() => setShowVoterFlow(true)}>
            Start voter flow
          </Button>
        </P>
      </Main>
    </Screen>
  );
}
