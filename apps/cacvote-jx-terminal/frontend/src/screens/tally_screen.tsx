import { assertDefined, iter } from '@votingworks/basics';
import { Button, H2, P } from '@votingworks/ui';
import { useParams } from 'react-router-dom';
import { format } from '@votingworks/utils';
import { useState } from 'react';
import * as api from '../api';
import { NavigationScreen } from './navigation_screen';
import { GenerateEncryptedTallyModal } from '../components/generate_encrypted_tally_modal';

export interface TallyScreenParams {
  electionId: string;
}

export function TallyScreen(): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const { electionId } = useParams<TallyScreenParams>();

  const [
    isShowingGenerateEncryptedTallyModal,
    setIsShowingGenerateEncryptedTallyModal,
  ] = useState(false);

  if (sessionData?.type !== 'authenticated' || !electionId) {
    return null;
  }

  const electionPresenter = assertDefined(
    sessionData.elections.find((e) => e.id === electionId)
  );

  const registeredVoterCount = iter(sessionData.registrations)
    .filter((r) => r.registration.electionObjectId === electionId)
    .count();

  const castBallotCount = iter(sessionData.castBallots)
    .filter((b) => b.registration.electionObjectId === electionId)
    .count();

  const isEncryptedElectionTallyPresent = false;
  const isDecryptedElectionTallyPresent = false;
  const isReadyToGenerateEncryptedTally = !isEncryptedElectionTallyPresent;
  const isReadyToDecryptElectionTally =
    isEncryptedElectionTallyPresent && !isDecryptedElectionTallyPresent;

  function onGenerateEncryptedTallyPressed() {
    setIsShowingGenerateEncryptedTallyModal(true);
  }

  function onGenerateEncryptedTallyConfirmed() {
    console.log('Generating encrypted tally');
  }

  function onExportEncryptedTallyPressed() {
    console.log('Exporting encrypted tally');
  }

  function onDecryptElectionTallyPressed() {
    console.log('Decrypting election tally');
  }

  function onExportDecryptedTallyPressed() {
    console.log('Exporting decrypted tally');
  }

  return (
    <NavigationScreen
      title="Tally Election"
      parentRoutes={[
        { title: 'Elections', path: '/elections' },
        {
          title: electionPresenter.election.electionDefinition.election.title,
          path: `/elections/${electionId}`,
        },
      ]}
    >
      <H2>Encrypted Election Tally</H2>
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
          onPress={onExportEncryptedTallyPressed}
          disabled={!isEncryptedElectionTallyPresent}
        >
          Export Encrypted Tally
        </Button>
      </P>

      <H2>Decrypted Election Tally</H2>
      <P>
        ElectionGuard may be used to decrypt the election tally. Once the tally
        is decrypted, the results can be exported.
      </P>
      <P>
        <Button
          icon="Bolt"
          onPress={onDecryptElectionTallyPressed}
          disabled={!isReadyToDecryptElectionTally}
        >
          Decrypt Election Tally
        </Button>
      </P>
      <P>
        <Button
          icon="Export"
          onPress={onExportDecryptedTallyPressed}
          disabled={!isDecryptedElectionTallyPresent}
        >
          Export Decrypted Tally
        </Button>
      </P>
      {isShowingGenerateEncryptedTallyModal && (
        <GenerateEncryptedTallyModal
          isGenerating={false}
          registeredVoterCount={registeredVoterCount}
          castBallotCount={castBallotCount}
          onGenerate={onGenerateEncryptedTallyConfirmed}
          onClose={() => setIsShowingGenerateEncryptedTallyModal(false)}
        />
      )}
    </NavigationScreen>
  );
}
