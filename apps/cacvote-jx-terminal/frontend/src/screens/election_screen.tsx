import { assertDefined } from '@votingworks/basics';
import { Button, H2, P, TD, TH, Table } from '@votingworks/ui';
import React from 'react';
import { useHistory, useParams } from 'react-router-dom';
import * as api from '../api';
import {
  AuthenticatedSessionData,
  CastBallotPresenter,
  RegistrationPresenter,
} from '../cacvote-server/session_data';
import { Uuid } from '../cacvote-server/types';
import { DateTimeCell } from '../components/date_time_cell';
import { ElectionCard } from '../components/election_card';
import { RegistrationConfigurationCell } from '../components/registration_configuration_cell';
import { VoterInfoCell } from '../components/voter_info_cell';
import { NavigationScreen } from './navigation_screen';

interface RegistrationsTableProps {
  registrations: readonly RegistrationPresenter[];
  castBallots: readonly CastBallotPresenter[];
}

function VotersAndBallotsTable({
  registrations,
  castBallots,
}: RegistrationsTableProps): JSX.Element {
  const castBallotsByRegistrationId = new Map<Uuid, CastBallotPresenter>();

  for (const castBallot of castBallots) {
    castBallotsByRegistrationId.set(castBallot.getRegistrationId(), castBallot);
  }

  return (
    <Table>
      <thead>
        <tr>
          <TH>Voter</TH>
          <TH>Election Configuration</TH>
          <TH>Registered</TH>
          <TH>Ballot Cast</TH>
        </tr>
      </thead>
      <tbody>
        {registrations.map((r) => {
          const castBallot = castBallotsByRegistrationId.get(r.getId());
          return (
            <tr key={r.getId()}>
              <VoterInfoCell
                displayName={r.getDisplayName()}
                commonAccessCardId={r.getRegistration().getCommonAccessCardId()}
              />
              <RegistrationConfigurationCell
                electionTitle={r.getElectionTitle()}
                ballotStyleId={r.getRegistration().getBallotStyleId()}
                precinctId={r.getRegistration().getPrecinctId()}
              />
              <DateTimeCell dateTime={r.getCreatedAt()} />
              {castBallot ? (
                <DateTimeCell dateTime={castBallot.getCreatedAt()} />
              ) : (
                <TD>
                  <P>No ballot cast</P>
                </TD>
              )}
            </tr>
          );
        })}
      </tbody>
    </Table>
  );
}

export interface ElectionScreenParams {
  electionId: string;
}

export function ElectionScreen(): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const isAuthenticated =
    sessionData && sessionData instanceof AuthenticatedSessionData;
  const { electionId } = useParams<ElectionScreenParams>();
  const history = useHistory();

  if (!isAuthenticated || !electionId) {
    return null;
  }

  const electionPresenter = assertDefined(
    sessionData.getElections().find((e) => e.getId() === electionId)
  );

  const registrations = sessionData
    .getRegistrations()
    .filter((r) => r.getRegistration().getElectionObjectId() === electionId);

  const castBallots = sessionData
    .getCastBallots()
    .filter((b) => b.getRegistration().getElectionObjectId() === electionId);

  function onPressTallyButton() {
    history.push(`/elections/${electionId}/tally`);
  }

  return (
    <NavigationScreen
      title={
        electionPresenter.getElection().getElectionDefinition().election.title
      }
      parentRoutes={[{ title: 'Elections', path: '/elections' }]}
    >
      <H2>Election Information</H2>
      <ElectionCard
        election={
          electionPresenter.getElection().getElectionDefinition().election
        }
      />

      <H2>Voters &amp; Ballots</H2>
      {registrations.length > 0 ? (
        <React.Fragment>
          <VotersAndBallotsTable
            registrations={registrations}
            castBallots={castBallots}
          />
          <br />
          <Button
            variant={
              registrations.length === castBallots.length
                ? 'primary'
                : 'neutral'
            }
            onPress={onPressTallyButton}
          >
            Go to Ballot Tally
          </Button>
        </React.Fragment>
      ) : (
        <P>No registered voters or cast ballots yet.</P>
      )}
    </NavigationScreen>
  );
}
