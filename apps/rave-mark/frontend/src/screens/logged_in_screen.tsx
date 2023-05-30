import { getVoterStatus } from '../api';
import { AdminFlowScreen } from './admin_flow_screen';
import { VoterFlowScreen } from './voter_flow_screen';

export function LoggedInScreen(): JSX.Element | null {
  const getVoterStatusQuery = getVoterStatus.useQuery();
  const voterStatus = getVoterStatusQuery.data;

  if (!voterStatus) {
    return null;
  }

  return voterStatus.isRaveAdmin ? <AdminFlowScreen /> : <VoterFlowScreen />;
}
