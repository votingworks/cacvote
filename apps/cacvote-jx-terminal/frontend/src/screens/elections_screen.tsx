import { Button, Card, H2, P, Seal } from '@votingworks/ui';
import { format } from '@votingworks/utils';
import { useState } from 'react';
import styled from 'styled-components';
import * as api from '../api';
import { NavigationScreen } from './navigation_screen';
import { CreateElectionModal } from '../components/create_election_modal';

const ElectionCard = styled(Card).attrs({ color: 'neutral' })`
  margin: 1rem 0;

  > div {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 1rem;
  }
`;

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
          // <Link to={`/elections/${id}`} key={id}>
          <ElectionCard key={id}>
            <Seal seal={election.seal} maxWidth="7rem" />
            <div>
              <H2 as="h3">{election.title}</H2>
              <P>
                {election.county.name}, {election.state}
                <br />
                {format.localeDate(new Date(election.date))}
              </P>
            </div>
          </ElectionCard>
          // </Link>
        )
      )}
      <Button icon="Add" onPress={() => setIsShowingAddElectionModal(true)}>
        Create New Election
      </Button>
      {isShowingAddElectionModal && (
        <CreateElectionModal
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
