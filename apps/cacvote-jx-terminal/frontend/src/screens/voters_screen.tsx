import { H2, P, TD, TH, Table } from '@votingworks/ui';
import { useState } from 'react';
import * as api from '../api';
import { RegistrationPresenter } from '../cacvote-server/session_data';
import { Uuid } from '../cacvote-server/types';
import { DateTimeCell } from '../components/date_time_cell';
import {
  ElectionConfiguration,
  ElectionConfigurationSelect,
} from '../components/election_configuration_select';
import { RegistrationConfigurationCell } from '../components/registration_configuration_cell';
import { VoterInfoCell } from '../components/voter_info_cell';
import { NavigationScreen } from './navigation_screen';

interface RegistrationsTableProps {
  registrations: readonly RegistrationPresenter[];
}

function RegistrationsTable({
  registrations,
}: RegistrationsTableProps): JSX.Element {
  return (
    <Table>
      <thead>
        <tr>
          <TH>Voter</TH>
          <TH>Election Configuration</TH>
          <TH>Registered</TH>
        </tr>
      </thead>
      <tbody>
        {registrations.map((r) => (
          <tr key={r.id}>
            <VoterInfoCell
              displayName={r.displayName}
              commonAccessCardId={r.registration.commonAccessCardId}
            />
            <RegistrationConfigurationCell
              electionTitle={r.electionTitle}
              ballotStyleId={r.registration.ballotStyleId}
              precinctId={r.registration.precinctId}
            />
            <DateTimeCell dateTime={r.createdAt} />
          </tr>
        ))}
      </tbody>
    </Table>
  );
}

export function VotersScreen(): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;

  const registerVoterMutation = api.registerVoter.useMutation();
  const [pendingRegistrationData, setPendingRegistrationData] = useState<{
    registrationRequestId: Uuid;
    electionConfiguration: ElectionConfiguration;
  }>();

  if (sessionData?.type !== 'authenticated') {
    return null;
  }

  return (
    <NavigationScreen title="Voters">
      <H2>Pending Registration Requests</H2>
      {sessionData.pendingRegistrationRequests.length > 0 ? (
        <Table>
          <thead>
            <tr>
              <TH>Voter</TH>
              <TH>Election Configuration</TH>
            </tr>
          </thead>
          <tbody>
            {sessionData.pendingRegistrationRequests.map((rr) => (
              <tr key={rr.id}>
                <TD>
                  <P>
                    {rr.displayName}
                    <br />
                    <em>CAC:</em> {rr.registrationRequest.commonAccessCardId}
                  </P>
                </TD>
                <TD>
                  <ElectionConfigurationSelect
                    elections={sessionData.elections.map((e) => ({
                      id: e.id,
                      election: e.election.electionDefinition.election,
                    }))}
                    value={
                      pendingRegistrationData?.registrationRequestId === rr.id
                        ? pendingRegistrationData.electionConfiguration
                        : undefined
                    }
                    onChange={(electionConfiguration) => {
                      if (electionConfiguration) {
                        setPendingRegistrationData({
                          registrationRequestId: rr.id,
                          electionConfiguration,
                        });

                        registerVoterMutation.mutate({
                          registrationRequestId: rr.id,
                          electionId: electionConfiguration.electionId,
                          ballotStyleId: electionConfiguration.ballotStyleId,
                          precinctId: electionConfiguration.precinctId,
                        });
                      }
                    }}
                    disabled={registerVoterMutation.isLoading}
                  />
                </TD>
              </tr>
            ))}
          </tbody>
        </Table>
      ) : (
        <P>There are no pending registration requests.</P>
      )}

      <H2>Registrations</H2>
      {sessionData.registrations.length > 0 ? (
        <RegistrationsTable registrations={sessionData.registrations} />
      ) : (
        <p>There are no registrations.</p>
      )}
    </NavigationScreen>
  );
}
