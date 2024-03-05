import { getVoterStatus } from '../api';
import { VoterFlowScreen } from './voter_flow_screen';

export function HasCardScreen(): JSX.Element | null {
  const getVoterStatusQuery = getVoterStatus.useQuery();
  const voterStatus = getVoterStatusQuery.data;

  if (!voterStatus) {
    return null;
  }

  return <VoterFlowScreen />;
}
