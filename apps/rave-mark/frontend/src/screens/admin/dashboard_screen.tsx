import { Button, H1, Main, P, Screen, Table } from '@votingworks/ui';
import { getServerSyncAttempts, sync } from '../../api';

export interface DashboardScreenProps {
  onClickShowVoterFlow: () => void;
}

export function DashboardScreen({
  onClickShowVoterFlow,
}: DashboardScreenProps): JSX.Element {
  const syncMutation = sync.useMutation();
  const getServerSyncAttemptsQuery = getServerSyncAttempts.useQuery();

  return (
    <Screen>
      <Main>
        <H1>Admin</H1>
        <P>
          <Button onPress={onClickShowVoterFlow}>Start voter flow</Button>
        </P>
        <Button
          onPress={() => syncMutation.mutate()}
          disabled={syncMutation.isLoading}
        >
          Sync with server
        </Button>
        <Table>
          <thead>
            <tr>
              <th>Started By</th>
              <th>Status</th>
              <th>Trigger</th>
              <th>Completed</th>
            </tr>
          </thead>
          <tbody>
            {getServerSyncAttemptsQuery.data?.map((attempt) => (
              <tr key={attempt.id}>
                <td>{attempt.creator}</td>
                <td>{attempt.statusMessage}</td>
                <td>{attempt.trigger}</td>
                <td>{attempt.completedAt?.toRelative()}</td>
              </tr>
            ))}
          </tbody>
        </Table>
      </Main>
    </Screen>
  );
}
