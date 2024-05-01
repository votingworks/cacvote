import { render, screen } from '@testing-library/react';
import { electionFamousNames2021Fixtures } from '@votingworks/fixtures';
import { unsafeParse } from '@votingworks/types';
import { Buffer } from 'buffer';
import { App } from './app';
import { AuthenticatedSessionData } from './cacvote-server/session_data';
import { JurisdictionCodeSchema, Uuid } from './cacvote-server/types';
import { mockEventSource } from '../test/mess';

let mess: ReturnType<typeof mockEventSource>;

beforeEach(() => {
  mess = mockEventSource();
  mess.install();
});

test('unauthenticated render', async () => {
  render(<App />);
  await screen.findByText(/Insert Your Card/i);
});

test('authenticated render', async () => {
  render(<App />);

  const jurisdictionCode = unsafeParse(
    JurisdictionCodeSchema,
    'st.test-jurisdiction'
  );
  const sessionData: AuthenticatedSessionData = {
    type: 'authenticated',
    jurisdictionCode,
    elections: [
      {
        id: Uuid(),
        election: {
          jurisdictionCode,
          mailingAddress: '123 Main St',
          electionDefinition:
            electionFamousNames2021Fixtures.electionDefinition,
          electionguardElectionMetadataBlob: Buffer.of(),
        },
      },
    ],
    pendingRegistrationRequests: [],
    registrations: [],
    castBallots: [],
  };

  mess.postMessage(
    '/api/status-stream',
    new MessageEvent('message', {
      // TODO: extract this serialization code somewhere
      data: JSON.stringify({
        ...sessionData,
        elections: sessionData.elections.map((election) => ({
          ...election,
          election: {
            ...election.election,
            electionDefinition: Buffer.from(
              election.election.electionDefinition.electionData
            ).toString('base64'),
            electionguardElectionMetadataBlob:
              election.election.electionguardElectionMetadataBlob.toString(
                'base64'
              ),
          },
        })),
      }),
    })
  );

  await screen.findByText(electionFamousNames2021Fixtures.election.title);
  await screen.findByText(/Create New Election/i);
});
