import { VotesDict } from '@votingworks/types';
import { Main, Prose, Screen, fontSizeTheme } from '@votingworks/ui';
import { useEffect } from 'react';
import { castBallot } from '../../api';
import { PinPadModal } from '../../components/pin_pad_modal';
import { COMMON_ACCESS_CARD_PIN_LENGTH } from '../../globals';

export interface SubmitScreenProps {
  votes: VotesDict;
  serialNumber: number;
  onSubmitSuccess: () => void;
  onCancel: () => void;
}

export function SubmitScreen({
  votes,
  serialNumber,
  onSubmitSuccess,
  onCancel,
}: SubmitScreenProps): JSX.Element {
  const castBallotMutation = castBallot.useMutation();
  const castBallotMutationMutate = castBallotMutation.mutate;
  const isCheckingPin = castBallotMutation.isLoading;

  function submitBallot(pin: string) {
    castBallotMutationMutate({ votes, serialNumber, pin });
  }

  useEffect(() => {
    if (castBallotMutation.data?.ok()) {
      onSubmitSuccess();
    }
  }, [castBallotMutation.data, onSubmitSuccess]);

  return (
    <Screen>
      <Main centerChild>
        <Prose
          textCenter
          themeDeprecated={fontSizeTheme.medium}
          maxWidth={false}
        >
          <PinPadModal
            pinLength={COMMON_ACCESS_CARD_PIN_LENGTH}
            primaryButtonLabel={
              isCheckingPin ? 'Checking…' : 'Confirm My Selections'
            }
            dismissButtonLabel="Go Back"
            onEnter={submitBallot}
            onDismiss={onCancel}
            disabled={isCheckingPin}
            error={castBallotMutation.data?.err()?.message}
          />
        </Prose>
      </Main>
    </Screen>
  );
}
