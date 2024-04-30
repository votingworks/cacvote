import { H2, P, TD, TH, Table } from '@votingworks/ui';
import { DateTime } from 'luxon';
import { useState } from 'react';
import * as api from '../api';
import { Uuid } from '../cacvote-server/types';
import {
  ElectionConfiguration,
  ElectionConfigurationSelect,
} from '../components/election_configuration_select';
import { NavigationScreen } from './navigation_screen';

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
        <Table>
          <thead>
            <tr>
              <TH>Voter</TH>
              <TH>Election Configuration</TH>
              <TH>Registered</TH>
            </tr>
          </thead>
          <tbody>
            {sessionData.registrations.map((r) => (
              <tr key={r.id}>
                <TD>
                  <P>{r.displayName}</P>
                  <P>
                    <em>CAC:</em> {r.registration.commonAccessCardId}
                  </P>
                </TD>
                <TD>
                  <P>{r.electionTitle}</P>
                  <P>
                    <em>Ballot Style:</em> {r.registration.ballotStyleId}
                    <br />
                    <em>Precinct:</em> {r.registration.precinctId}
                  </P>
                </TD>
                <TD>
                  <P>
                    {DateTime.fromISO(r.createdAt).toLocaleString(
                      DateTime.DATETIME_SHORT
                    )}
                  </P>
                </TD>
              </tr>
            ))}
          </tbody>
        </Table>
      ) : (
        <p>There are no registrations.</p>
      )}
    </NavigationScreen>
  );
}
