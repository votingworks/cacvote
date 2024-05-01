import { Button } from '@votingworks/ui';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import * as api from '../api';
import { CreateElectionModal } from '../components/create_election_modal';
import { ElectionCard } from '../components/election_card';
import { NavigationScreen } from './navigation_screen';

export function ElectionsScreen(): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const createElectionMutation = api.createElection.useMutation();

  const [isShowingAddElectionModal, setIsShowingAddElectionModal] =
    useState(false);

  if (sessionData?.type !== 'authenticated') {
    return null;
  }

  return (
    <NavigationScreen title="Elections">
      {sessionData.elections.map(
        ({
          id,
          election: {
            electionDefinition: { election },
          },
        }) => (
          <Link to={`/elections/${id}`} key={id}>
            <ElectionCard election={election} />
          </Link>
        )
      )}
      <Button icon="Add" onPress={() => setIsShowingAddElectionModal(true)}>
        Create New Election
      </Button>
      {isShowingAddElectionModal && (
        <CreateElectionModal
          isCreating={createElectionMutation.isLoading}
          onCreate={({ mailingAddress, electionDefinition }) =>
            createElectionMutation.mutate(
              {
                jurisdictionCode: sessionData.jurisdictionCode,
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
