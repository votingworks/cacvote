import { useState } from 'react';
import { VoterFlowScreen } from './voter_flow_screen';
import * as Admin from './admin';

export function AdminFlowScreen(): JSX.Element {
  const [showVoterFlow, setShowVoterFlow] = useState(false);

  if (showVoterFlow) {
    return <VoterFlowScreen />;
  }

  return (
    <Admin.DashboardScreen
      onClickShowVoterFlow={() => setShowVoterFlow(true)}
    />
  );
}
