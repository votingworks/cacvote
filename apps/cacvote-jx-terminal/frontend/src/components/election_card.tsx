import { Election } from '@votingworks/types';
import { Card, H2, P, Seal } from '@votingworks/ui';
import { format } from '@votingworks/utils';
import styled from 'styled-components';

const Wrapper = styled(Card).attrs({ color: 'neutral' })`
  margin: 1rem 0;

  > div {
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 1rem;
  }
`;

export interface ElectionCardProps {
  election: Election;
}

export function ElectionCard({ election }: ElectionCardProps): JSX.Element {
  return (
    <Wrapper>
      <Seal seal={election.seal} maxWidth="7rem" />
      <div>
        <H2 as="h3">{election.title}</H2>
        <P>
          {election.county.name}, {election.state}
          <br />
          {format.localeDate(new Date(election.date))}
        </P>
      </div>
    </Wrapper>
  );
}
