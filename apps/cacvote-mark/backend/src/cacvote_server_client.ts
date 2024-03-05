/* eslint-disable max-classes-per-file */

import { VX_MACHINE_ID } from '@votingworks/backend';
import { assert } from '@votingworks/basics';
import { unsafeParse } from '@votingworks/types';
import { format } from '@votingworks/utils';
import { Buffer } from 'buffer';
import fetch from 'cross-fetch';
import makeDebug from 'debug';
import { Store } from './store';
import { AuthStatus } from './types/auth';
import { ClientId } from './types/db';
import {
  RaveMarkSyncOutputSchema,
  RaveServerSyncInput,
  RaveServerSyncOutput,
} from './types/sync';

const debug = makeDebug('cacvote-server-client');

export interface RaveServerClient {
  sync(options?: { authStatus: AuthStatus }): Promise<void>;
}

function describeSyncInputOrOutput(
  data: RaveServerSyncInput | RaveServerSyncOutput
): string[] {
  const parts: string[] = [];

  if (data.printedBallots?.length) {
    parts.push(
      format.countPhrase({
        value: data.printedBallots.length,
        one: '1 printed ballot',
        many: `${data.printedBallots.length} printed ballots`,
      })
    );
  }

  if (data.jurisdictions?.length) {
    parts.push(
      format.countPhrase({
        value: data.jurisdictions.length,
        one: '1 jurisdiction',
        many: `${data.jurisdictions.length} jurisdictions`,
      })
    );
  }

  if (data.elections?.length) {
    parts.push(
      format.countPhrase({
        value: data.elections.length,
        one: '1 election',
        many: `${data.elections.length} elections`,
      })
    );
  }

  if (data.registrationRequests?.length) {
    parts.push(
      format.countPhrase({
        value: data.registrationRequests.length,
        one: '1 registration request',
        many: `${data.registrationRequests.length} registration requests`,
      })
    );
  }

  if (data.registrations?.length) {
    parts.push(
      format.countPhrase({
        value: data.registrations.length,
        one: '1 registration',
        many: `${data.registrations.length} registrations`,
      })
    );
  }

  return parts;
}

export class RaveServerClientImpl {
  private readonly store: Store;
  private readonly baseUrl: URL;

  constructor({ store, baseUrl }: { store: Store; baseUrl: URL }) {
    this.store = store;
    this.baseUrl = baseUrl;
  }

  async sync({ authStatus }: { authStatus?: AuthStatus } = {}): Promise<void> {
    const user = authStatus?.status === 'has_card' ? authStatus.card : null;
    assert(!authStatus || user, 'not logged in as voter');
    const creator = user?.commonAccessCardId ?? 'system';

    const syncAttemptId = this.store.createServerSyncAttempt({
      creator,
      trigger: user ? 'manual' : 'scheduled',
      initialStatusMessage: 'Syncing…',
    });

    try {
      const input = this.createSyncInput();
      const output = await this.performSyncRequest(input);
      this.updateLocalStoreFromSyncOutput(output);
      this.updateServerSyncAttempt({ syncAttemptId, input, output });
    } catch (error) {
      debug(
        'CACVote Server sync failed: %s',
        error instanceof Error ? error.stack : error
      );
      this.store.updateServerSyncAttempt({
        id: syncAttemptId,
        status: 'failure',
        statusMessage:
          error instanceof Error ? error.message : `unknown error: ${error}`,
      });
    }
  }

  private createSyncInput(): RaveServerSyncInput {
    const input: RaveServerSyncInput = {
      lastSyncedRegistrationRequestId:
        this.store.getLastSyncedRegistrationRequestId(),
      lastSyncedRegistrationId: this.store.getLastSyncedRegistrationId(),
      lastSyncedElectionId: this.store.getLastSyncedElectionId(),
      lastSyncedPrintedBallotId: this.store.getLastSyncedPrintedBallotId(),
      registrationRequests: this.store.getRegistrationRequestsToSync(),
      jurisdictions: this.store.getJurisdictionsToSync(),
      elections: this.store.getElectionsToSync(),
      registrations: this.store.getRegistrationsToSync(),
      printedBallots: this.store.getPrintedBallotsToSync(),
    };
    debug('CACVote sync input: %O', input);
    return input;
  }

  private async performSyncRequest(
    input: RaveServerSyncInput
  ): Promise<RaveServerSyncOutput> {
    const syncUrl = new URL('api/sync', this.baseUrl);
    const response = await fetch(syncUrl.toString(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(input),
    });
    if (response.status !== 200) {
      const text = await response.text();
      throw new Error(
        `Server responded with ${response.status} ${response.statusText}: ${text}}`
      );
    }

    return this.parseSyncOutput(await response.json());
  }

  private parseSyncOutput(rawOutput: unknown): RaveServerSyncOutput {
    debug('CACVote sync raw output: %O', rawOutput);
    const output = unsafeParse(RaveMarkSyncOutputSchema, rawOutput);
    debug('CACVote sync parsed output: %O', output);
    return output;
  }

  private updateLocalStoreFromSyncOutput(output: RaveServerSyncOutput): void {
    for (const jurisdiction of output.jurisdictions) {
      const jurisdictionId = this.store.createJurisdiction(jurisdiction);
      debug('created or replaced jurisdiction %s', jurisdictionId);
    }

    for (const election of output.elections) {
      const electionId = this.store.createElection({
        id:
          election.machineId === VX_MACHINE_ID ? election.clientId : ClientId(),
        ...election,
        definition: Buffer.from(election.definition, 'base64'),
      });
      debug('created or replaced election %s', electionId);
    }

    for (const registrationRequest of output.registrationRequests) {
      const registrationRequestId = this.store.createRegistrationRequest({
        id:
          registrationRequest.machineId === VX_MACHINE_ID
            ? registrationRequest.clientId
            : ClientId(),
        ...registrationRequest,
      });
      debug(
        'created or replaced registration request %s',
        registrationRequestId
      );
    }

    for (const registration of output.registrations) {
      const localRegistrationRequest = this.store.getRegistrationRequest({
        serverId: registration.registrationRequestId,
      });
      assert(
        localRegistrationRequest,
        `could not find local registration request with server id ${registration.registrationRequestId}`
      );

      const localElection = this.store.getElection({
        serverId: registration.electionId,
      });
      assert(
        localElection,
        `could not find local election with server id ${registration.electionId}`
      );

      const registrationId = this.store.createRegistration({
        id:
          registration.machineId === VX_MACHINE_ID
            ? registration.clientId
            : ClientId(),
        serverId: registration.serverId,
        clientId: registration.clientId,
        machineId: registration.machineId,
        jurisdictionId: registration.jurisdictionId,
        registrationRequestId: localRegistrationRequest.id,
        electionId: localElection.id,
        ballotStyleId: registration.ballotStyleId,
        precinctId: registration.precinctId,
      });

      debug('created or replaced registration %s', registrationId);
    }

    for (const printedBallot of output.printedBallots) {
      const localRegistration = this.store.getRegistration({
        serverId: printedBallot.registrationId,
      });

      assert(
        localRegistration,
        `could not find local registration with server id ${printedBallot.registrationId}`
      );

      const ballotId = this.store.createPrintedBallot({
        id:
          printedBallot.machineId === VX_MACHINE_ID
            ? printedBallot.clientId
            : ClientId(),
        serverId: printedBallot.serverId,
        clientId: printedBallot.clientId,
        machineId: printedBallot.machineId,
        registrationId: localRegistration.clientId,
        commonAccessCardCertificate: Buffer.from(
          printedBallot.commonAccessCardCertificate,
          'base64'
        ),
        castVoteRecord: Buffer.from(printedBallot.castVoteRecord, 'base64'),
        castVoteRecordSignature: Buffer.from(
          printedBallot.castVoteRecordSignature,
          'base64'
        ),
      });

      debug('created or replaced printed ballot %s', ballotId);
    }
  }

  private updateServerSyncAttempt({
    syncAttemptId,
    input,
    output,
  }: {
    syncAttemptId: ClientId;
    input: RaveServerSyncInput;
    output: RaveServerSyncOutput;
  }): void {
    const sentParts = describeSyncInputOrOutput(input);
    const receivedParts = describeSyncInputOrOutput(output);

    this.store.updateServerSyncAttempt({
      id: syncAttemptId,
      status: 'success',
      statusMessage: `SENT: ${
        sentParts.length === 0 ? 'nothing' : sentParts.join(', ')
      }\nRECEIVED: ${
        receivedParts.length === 0 ? 'nothing' : receivedParts.join(', ')
      }`,
    });
  }
}

export class MockRaveServerClient implements RaveServerClient {
  constructor(private readonly store: Store) {}

  async sync({
    authStatus,
  }: {
    authStatus?: AuthStatus;
  } = {}): Promise<void> {
    await Promise.resolve();

    const card = authStatus?.status === 'has_card' ? authStatus.card : null;
    assert(!authStatus || card, 'not logged in as voter');
    const creator = card?.commonAccessCardId ?? 'system';

    const id = this.store.createServerSyncAttempt({
      creator,
      trigger: card ? 'manual' : 'scheduled',
      initialStatusMessage: 'Syncing…',
    });

    setTimeout(() => {
      this.store.updateServerSyncAttempt({
        id,
        status: 'success',
        statusMessage: `SENT: nothing\nRECEIVED: nothing`,
      });
    }, 1000);
  }
}
