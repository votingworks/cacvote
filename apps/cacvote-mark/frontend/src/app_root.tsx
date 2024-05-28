import { useState } from 'react';
import * as api from './api';
import { WelcomeScreen } from './screens/welcome_screen';
import { VoterFlowScreen } from './screens/voter_flow_screen';

export function AppRoot(): JSX.Element {
  const [isVoterSessionStillActive, setIsVoterSessionStillActive] =
    useState(false);
  const getVoterStatusQuery = api.getVoterStatus.useQuery();

  if (isVoterSessionStillActive || getVoterStatusQuery.data?.status) {
    return (
      <VoterFlowScreen
        setIsVoterSessionStillActive={setIsVoterSessionStillActive}
      />
    );
  }

  return <WelcomeScreen />;
}
