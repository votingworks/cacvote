import { render, screen } from '@testing-library/react';
import { electionFamousNames2021Fixtures } from '@votingworks/fixtures';
import { Mess, mockEventSource } from '@votingworks/test-utils';
import { unsafeParse } from '@votingworks/types';
import { Buffer } from 'buffer';
import { App } from './app';
import {
  AuthenticatedSessionData,
  ElectionInfo,
  ElectionPresenter,
} from './cacvote-server/session_data';
import { JurisdictionCodeSchema, Uuid } from './cacvote-server/types';

let mess: Mess;

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
  const sessionData = new AuthenticatedSessionData(
    jurisdictionCode,
    [
      new ElectionPresenter(
        Uuid(),
        new ElectionInfo(
          jurisdictionCode,
          electionFamousNames2021Fixtures.electionDefinition,
          '123 Main St',
          Buffer.of()
        )
      ),
    ],
    [],
    [],
    []
  );

  mess.postMessage(
    '/api/status-stream',
    new MessageEvent('message', {
      data: JSON.stringify(sessionData),
    })
  );

  await screen.findByText(electionFamousNames2021Fixtures.election.title);
  await screen.findByText(/Create New Election/i);
});
