import { assert, assertDefined, iter } from '@votingworks/basics';
import { Button, H2, LoadingButton, P } from '@votingworks/ui';
import { format } from '@votingworks/utils';
import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { DateTime } from 'luxon';
import * as api from '../api';
import { Uuid } from '../cacvote-server/types';
import { GenerateEncryptedTallyModal } from '../components/generate_encrypted_tally_modal';
import { NavigationScreen } from './navigation_screen';
import { downloadData } from '../utils/download';
import { AuthenticatedSessionData } from '../cacvote-server/session_data';

export interface TallyScreenParams {
  electionId: Uuid;
}

function ObjectStatus({
  createdAt,
  postedAt,
}: {
  createdAt?: DateTime;
  postedAt?: DateTime;
}): JSX.Element {
  return (
    <P>
      <strong>Created:</strong>{' '}
      {createdAt?.toLocaleString(DateTime.DATETIME_SHORT) ?? 'n/a'}
      <br />
      <strong>Posted:</strong>{' '}
      {postedAt?.toLocaleString(DateTime.DATETIME_SHORT) ?? 'n/a'}
    </P>
  );
}

export function TallyScreen(): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const isAuthenticated =
    sessionData && sessionData instanceof AuthenticatedSessionData;
  const { electionId } = useParams<TallyScreenParams>();

  const [
    isShowingGenerateEncryptedTallyModal,
    setIsShowingGenerateEncryptedTallyModal,
  ] = useState(false);
  const generateEncryptedElectionTallyMutation =
    api.generateEncryptedElectionTally.useMutation();
  const decryptEncryptedElectionTallyMutation =
    api.decryptEncryptedElectionTally.useMutation();

  if (!isAuthenticated || !electionId) {
    return null;
  }

  const electionPresenter = assertDefined(
    sessionData.getElections().find((e) => e.getId() === electionId)
  );

  const registeredVoterCount = iter(sessionData.getRegistrations())
    .filter((r) => r.getRegistration().getElectionObjectId() === electionId)
    .count();

  const castBallotCount = iter(sessionData.getCastBallots())
    .filter((b) => b.getRegistration().getElectionObjectId() === electionId)
    .count();

  const isEncryptedElectionTallyPresent = Boolean(
    electionPresenter.getEncryptedTally()
  );
  const isDecryptedElectionTallyPresent = Boolean(
    electionPresenter.getDecryptedTally()
  );
  const isReadyToGenerateEncryptedTally = !isEncryptedElectionTallyPresent;
  const isReadyToDecryptElectionTally =
    isEncryptedElectionTallyPresent && !isDecryptedElectionTallyPresent;

  function onGenerateEncryptedTallyPressed() {
    setIsShowingGenerateEncryptedTallyModal(true);
  }

  async function onGenerateEncryptedTallyConfirmed() {
    await generateEncryptedElectionTallyMutation.mutateAsync({ electionId });
  }

  function onSaveEncryptedTallyPressed() {
    const encryptedTally = electionPresenter.getEncryptedTally();
    assert(encryptedTally);
    downloadData(
      encryptedTally
        .getEncryptedElectionTally()
        .getElectionguardEncryptedTally(),
      `encrypted-tally-${electionId}.json`
    );
  }

  function onDecryptElectionTallyPressed() {
    decryptEncryptedElectionTallyMutation.mutate({ electionId });
  }

  function onSaveDecryptedTallyPressed() {
    const decryptedTally = electionPresenter.getDecryptedTally();
    assert(decryptedTally);
    downloadData(
      decryptedTally
        .getDecryptedElectionTally()
        .getElectionguardDecryptedTally(),
      `decrypted-tally-${electionId}.json`
    );
  }

  return (
    <NavigationScreen
      title="Tally Election"
      parentRoutes={[
        { title: 'Elections', path: '/elections' },
        {
          title: electionPresenter.getElection().getElectionDefinition()
            .election.title,
          path: `/elections/${electionId}`,
        },
      ]}
    >
      <H2>Encrypted Election Tally</H2>
      <ObjectStatus
        createdAt={electionPresenter.getEncryptedTally()?.getCreatedAt()}
        postedAt={electionPresenter.getEncryptedTally()?.getSyncedAt()}
      />
      <P>
        <strong>Registered voter count:</strong>{' '}
        {format.count(registeredVoterCount)}
      </P>
      <P>
        <strong>Encrypted cast ballot count:</strong>{' '}
        {format.count(castBallotCount)}
      </P>
      <P>
        You may generate an encrypted tally of all encrypted cast ballots in
        this election. This operation may only be performed once, and should be
        performed only once all ballots to be counted are present in the system.
      </P>
      <P>
        <Button
          icon="Add"
          onPress={onGenerateEncryptedTallyPressed}
          disabled={!isReadyToGenerateEncryptedTally}
        >
          Generate Encrypted Tally
        </Button>
      </P>
      <P>
        <Button
          icon="Export"
          onPress={onSaveEncryptedTallyPressed}
          disabled={!isEncryptedElectionTallyPresent}
        >
          Save Encrypted Tally
        </Button>
      </P>

      <H2>Decrypted Election Tally</H2>
      <ObjectStatus
        createdAt={electionPresenter.getDecryptedTally()?.getCreatedAt()}
        postedAt={electionPresenter.getDecryptedTally()?.getSyncedAt()}
      />
      <P>
        This operation uses ElectionGuard to decrypt only the encrypted tally,
        not any of the encrypted cast ballots. Once the tally is decrypted, the
        decrypted tally will automatically be posted to the bulletin board. The
        decrypted tally can be saved below.
      </P>
      <P>
        {decryptEncryptedElectionTallyMutation.isLoading ? (
          <LoadingButton>Decrypting Election Tally…</LoadingButton>
        ) : (
          <Button
            icon="Unlock"
            onPress={onDecryptElectionTallyPressed}
            disabled={!isReadyToDecryptElectionTally}
          >
            Decrypt Election Tally
          </Button>
        )}
      </P>
      <P>
        <Button
          icon="Export"
          onPress={onSaveDecryptedTallyPressed}
          disabled={!isDecryptedElectionTallyPresent}
        >
          Save Decrypted Tally
        </Button>
      </P>
      {isShowingGenerateEncryptedTallyModal && (
        <GenerateEncryptedTallyModal
          isGenerating={generateEncryptedElectionTallyMutation.isLoading}
          registeredVoterCount={registeredVoterCount}
          castBallotCount={castBallotCount}
          onGenerate={onGenerateEncryptedTallyConfirmed}
          onClose={() => setIsShowingGenerateEncryptedTallyModal(false)}
        />
      )}
    </NavigationScreen>
  );
}
