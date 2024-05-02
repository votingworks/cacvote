import { Button } from '@votingworks/ui';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import * as api from '../api';
import { AuthenticatedSessionData } from '../cacvote-server/session_data';
import { CreateElectionModal } from '../components/create_election_modal';
import { ElectionCard } from '../components/election_card';
import { NavigationScreen } from './navigation_screen';

export function ElectionsScreen(): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const isAuthenticated =
    sessionData && sessionData instanceof AuthenticatedSessionData;
  const createElectionMutation = api.createElection.useMutation();

  const [isShowingAddElectionModal, setIsShowingAddElectionModal] =
    useState(false);

  if (!isAuthenticated) {
    return null;
  }

  return (
    <NavigationScreen title="Elections">
      {sessionData.getElections().map((e) => (
        <Link to={`/elections/${e.getId()}`} key={e.getId()}>
          <ElectionCard
            election={e.getElection().getElectionDefinition().election}
          />
        </Link>
      ))}
      <Button icon="Add" onPress={() => setIsShowingAddElectionModal(true)}>
        Create New Election
      </Button>
      {isShowingAddElectionModal && (
        <CreateElectionModal
          isCreating={createElectionMutation.isLoading}
          onCreate={({ mailingAddress, electionDefinition }) =>
            createElectionMutation.mutate(
              {
                jurisdictionCode: sessionData.getJurisdictionCode(),
                electionDefinition,
                mailingAddress,
              },
              {
                onSuccess: () => {
                  setIsShowingAddElectionModal(false);
                },
              }
            )
          }
          onClose={() => setIsShowingAddElectionModal(false)}
        />
      )}
    </NavigationScreen>
  );
}
