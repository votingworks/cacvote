import { VotesDict } from '@votingworks/types';
import {
  Button,
  H2,
  Main,
  P,
  Prose,
  Screen,
  fontSizeTheme,
} from '@votingworks/ui';
import { useState } from 'react';
import { castBallot } from '../../api';
import { PinPadModal } from '../../components/pin_pad_modal';
import { COMMON_ACCESS_CARD_PIN_LENGTH } from '../../globals';

export interface SubmitScreenProps {
  votes: VotesDict;
  onSubmitted: () => void;
}

export function SubmitScreen({
  votes,
  onSubmitted,
}: SubmitScreenProps): JSX.Element {
  const [pinError, setPinError] = useState<string>();
  const [isShowingPinModal, setIsShowingPinModal] = useState(false);
  const castBallotMutation = castBallot.useMutation();
  const castBallotMutationMutateAsync = castBallotMutation.mutateAsync;
  const isCheckingPin = castBallotMutation.isLoading;

  async function submitBallot(pin: string) {
    setPinError(undefined);
    try {
      await castBallotMutationMutateAsync(
        { votes, pin },
        {
          onError(err) {
            setPinError(err instanceof Error ? err.message : `${err}`);
          },
        }
      );
      onSubmitted();
    } catch (err) {
      setPinError(err instanceof Error ? err.message : `${err}`);
    }
  }

  async function handleEnter(pin: string) {
    await submitBallot(pin);
  }

  return (
    <Screen>
      <Main centerChild>
        <Prose
          textCenter
          themeDeprecated={fontSizeTheme.medium}
          maxWidth={false}
        >
          <H2>You’re Almost Done</H2>
          <P>
            Thanks for reviewing your printed ballot!
            <br />
            Tap the button below to continue.
          </P>
          <Button variant="primary" onPress={() => setIsShowingPinModal(true)}>
            Cast My Ballot
          </Button>
          {isShowingPinModal && (
            <PinPadModal
              pinLength={COMMON_ACCESS_CARD_PIN_LENGTH}
              primaryButtonLabel={
                isCheckingPin ? 'Checking…' : 'Cast My Ballot'
              }
              dismissButtonLabel="Go Back"
              onEnter={handleEnter}
              onDismiss={() => setIsShowingPinModal(false)}
              disabled={isCheckingPin}
              error={pinError}
            />
          )}
        </Prose>
      </Main>
    </Screen>
  );
}
