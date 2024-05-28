import { BallotStyleId, PrecinctId } from '@votingworks/types';
import { P, TD } from '@votingworks/ui';

export interface RegistrationsConfigurationCellProps {
  electionTitle: string;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
}

export function RegistrationConfigurationCell({
  electionTitle,
  ballotStyleId,
  precinctId,
}: RegistrationsConfigurationCellProps): JSX.Element {
  return (
    <TD>
      <P>{electionTitle}</P>
      <P>
        <em>Ballot Style:</em> {ballotStyleId}
        <br />
        <em>Precinct:</em> {precinctId}
      </P>
    </TD>
  );
}
