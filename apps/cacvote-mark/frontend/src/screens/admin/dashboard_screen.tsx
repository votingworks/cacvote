import {
  Button,
  H1,
  H2,
  Main,
  Screen,
  TD,
  TH,
  Table,
  Text,
} from '@votingworks/ui';
import React from 'react';
import styled from 'styled-components';
import { format } from '@votingworks/utils';
import { getServerSyncStatus, sync } from '../../api';

export interface DashboardScreenProps {
  onClickShowVoterFlow: () => void;
}

const FloatingTopRight = styled.div`
  position: absolute;
  top: 0;
  right: 0;
  padding: 1rem;
  display: flex;
  flex-direction: row;
  gap: 1rem;
`;

export function DashboardScreen({
  onClickShowVoterFlow,
}: DashboardScreenProps): JSX.Element {
  const syncMutation = sync.useMutation();
  const getServerSyncStatusQuery = getServerSyncStatus.useQuery();

  return (
    <Screen>
      <Main>
        <H1>Admin</H1>
        <FloatingTopRight>
          <Button onPress={onClickShowVoterFlow} icon="Previous">
            Exit to Voter Flow
          </Button>
          <Button
            onPress={() => syncMutation.mutate()}
            disabled={syncMutation.isLoading}
          >
            Start Sync Manually
          </Button>
        </FloatingTopRight>

        <H2>Sync Status</H2>
        <Table>
          <thead>
            <tr>
              <TH>Type</TH>
              <TH>Synced with Server</TH>
              <TH>Pending Sync to Server</TH>
            </tr>
          </thead>
          <tbody>
            <tr>
              <TD>Ballots Cast</TD>
              <TD>
                {format.count(
                  getServerSyncStatusQuery.data?.status.printedBallots.synced ??
                    0
                )}
              </TD>
              <TD>
                {format.count(
                  getServerSyncStatusQuery.data?.status.printedBallots
                    .pending ?? 0
                )}
              </TD>
            </tr>
            <tr>
              <TD>Registration Requests</TD>
              <TD>
                {format.count(
                  getServerSyncStatusQuery.data?.status.pendingRegistrations
                    .synced ?? 0
                )}
              </TD>
              <TD>
                {format.count(
                  getServerSyncStatusQuery.data?.status.pendingRegistrations
                    .pending ?? 0
                )}
              </TD>
            </tr>
            <tr>
              <TD>Elections</TD>
              <TD>
                {format.count(
                  getServerSyncStatusQuery.data?.status.elections.synced ?? 0
                )}
              </TD>
              <TD>n/a</TD>
            </tr>
          </tbody>
        </Table>

        <H2>Sync History</H2>
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
            {getServerSyncStatusQuery.data?.attempts.map((attempt) => (
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
