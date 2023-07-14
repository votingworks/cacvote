import { Button, H1, Main, P, Screen, Table, Text } from '@votingworks/ui';
import React from 'react';
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
                <td>
                  <Text small>
                    {attempt.statusMessage.split('\n').map((line, i, lines) => (
                      <React.Fragment key={line}>
                        {line}
                        {i < lines.length - 1 && <br />}
                      </React.Fragment>
                    ))}
                  </Text>
                </td>
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
