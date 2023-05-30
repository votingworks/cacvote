import { Review as MarkFlowReview } from '@votingworks/mark-flow-ui';
import { Button, H1, Main, Screen, WithScrollButtons } from '@votingworks/ui';
import styled from 'styled-components';
import {
  Contests,
  ElectionDefinition,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import { ButtonFooter } from '../../components/button_footer';

const ContentHeader = styled.div`
  padding: 0.5rem 0.75rem 0;
`;

export interface ReviewScreenProps {
  electionDefinition: ElectionDefinition;
  contests: Contests;
  votes: VotesDict;
  precinctId: PrecinctId;
  onConfirm: () => void;
  goToIndex: (contestIndex: number) => void;
}

export function ReviewScreen({
  electionDefinition,
  contests,
  votes,
  precinctId,
  onConfirm,
  goToIndex,
}: ReviewScreenProps): JSX.Element {
  return (
    <Screen>
      <Main flexColumn>
        <ContentHeader>
          <H1>Review Your Votes</H1>
        </ContentHeader>
        <WithScrollButtons>
          <MarkFlowReview
            election={electionDefinition.election}
            contests={contests}
            precinctId={precinctId}
            votes={votes}
            returnToContest={(contestId) => {
              goToIndex(contests.findIndex((c) => c.id === contestId));
            }}
          />
        </WithScrollButtons>
      </Main>
      <ButtonFooter>
        <Button onPress={onConfirm} variant="primary">
          Cast My Ballot
        </Button>
      </ButtonFooter>
    </Screen>
  );
}
