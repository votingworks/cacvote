import { Uuid } from '@votingworks/cacvote-mark-backend';
import { VotesDict } from '@votingworks/types';
import { Main, Prose, Screen, fontSizeTheme } from '@votingworks/ui';
import { useEffect } from 'react';
import { castBallot } from '../../api';
import { PinPadModal } from '../../components/pin_pad_modal';
import { COMMON_ACCESS_CARD_PIN_LENGTH } from '../../globals';

export interface SubmitScreenProps {
  votes: VotesDict;
  serialNumber: number;
  onSubmitSuccess: (castBallotObjectId: Uuid) => void;
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
    const castBallotResult = castBallotMutation.data;
    if (castBallotResult?.isOk()) {
      onSubmitSuccess(castBallotResult.ok().id);
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
