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
import { useEffect, useState } from 'react';
import { castBallot } from '../../api';
import { PinPadModal } from '../../components/pin_pad_modal';
import { COMMON_ACCESS_CARD_PIN_LENGTH } from '../../globals';

export interface SubmitScreenProps {
  votes: VotesDict;
  serialNumber: number;
  onSubmitted: () => void;
}

export function SubmitScreen({
  votes,
  serialNumber,
  onSubmitted,
}: SubmitScreenProps): JSX.Element {
  const [isShowingPinModal, setIsShowingPinModal] = useState(false);
  const castBallotMutation = castBallot.useMutation();
  const castBallotMutationMutate = castBallotMutation.mutate;
  const isCheckingPin = castBallotMutation.isLoading;

  function submitBallot(pin: string) {
    castBallotMutationMutate({ votes, serialNumber, pin });
  }

  useEffect(() => {
    if (castBallotMutation.data?.ok()) {
      onSubmitted();
    }
  }, [castBallotMutation.data, onSubmitted]);

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
              onEnter={submitBallot}
              onDismiss={() => setIsShowingPinModal(false)}
              disabled={isCheckingPin}
              error={castBallotMutation.data?.err()?.message}
            />
          )}
        </Prose>
      </Main>
    </Screen>
  );
}
