import {
  BallotStyleId,
  Election,
  NewType,
  PrecinctId,
} from '@votingworks/types';
import { SearchSelect, SelectOption } from '@votingworks/ui';
import { Uuid } from '../cacvote-server/types';

export interface ElectionConfiguration {
  electionId: Uuid;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
}

export interface ElectionConfigurationSelectProps {
  elections: Array<{ id: Uuid; election: Election }>;
  value?: ElectionConfiguration;
  onChange: (configuration?: ElectionConfiguration) => void;
  disabled?: boolean;
}

type ElectionConfigurationId = NewType<string, 'ElectionConfigurationId'>;

function encodeElectionConfiguration(
  electionConfiguration: ElectionConfiguration
): ElectionConfigurationId {
  return JSON.stringify(electionConfiguration) as ElectionConfigurationId;
}

function decodeElectionConfiguration(
  electionConfigurationId: ElectionConfigurationId
): ElectionConfiguration {
  return JSON.parse(electionConfigurationId) as ElectionConfiguration;
}

export function ElectionConfigurationSelect({
  elections,
  value,
  onChange,
  disabled,
}: ElectionConfigurationSelectProps): JSX.Element {
  const options: Array<SelectOption<ElectionConfigurationId>> =
    elections.flatMap(({ id, election }) =>
      election.ballotStyles.flatMap((bs) =>
        bs.precincts.map((precinctId) => ({
          value: encodeElectionConfiguration({
            electionId: id,
            ballotStyleId: bs.id,
            precinctId,
          }),
          label: `${election.title} - BS: ${bs.id} - P: ${precinctId}`,
        }))
      )
    );

  return (
    <SearchSelect
      placeholder="Select an election configuration"
      value={value ? encodeElectionConfiguration(value) : undefined}
      disabled={disabled}
      options={options}
      onChange={(newValue) =>
        onChange(newValue ? decodeElectionConfiguration(newValue) : undefined)
      }
    />
  );
}
