import { Id, VotesDict } from '@votingworks/types';
import {
  Button,
  Main,
  NumberPad,
  P,
  Prose,
  Screen,
  fontSizeTheme,
  usePinEntry,
} from '@votingworks/ui';
import { useCallback, useState } from 'react';
import styled from 'styled-components';
import { createBallotPendingPrint } from '../../api';
import { COMMON_ACCESS_CARD_PIN_LENGTH } from '../../globals';

const NumberPadWrapper = styled.div`
  display: flex;
  justify-content: center;
  margin-top: 10px;
  font-size: 1em;
  > div {
    width: 400px;
  }
  *:focus {
    outline: none;
  }
`;

const EnteredCode = styled.div`
  margin-top: 5px;
  text-align: center;
  font-family: monospace;
  font-size: 1.5em;
  font-weight: 600;
`;

export interface SubmitScreenProps {
  votes: VotesDict;
  onSubmitted: (ballotPendingPrintId: Id) => void;
}

const pinLength = COMMON_ACCESS_CARD_PIN_LENGTH;

export function SubmitScreen({
  votes,
  onSubmitted,
}: SubmitScreenProps): JSX.Element {
  const [error, setError] = useState<string>();
  const [isCheckingPin, setIsCheckingPin] = useState(false);
  const pinEntry = usePinEntry({ pinLength });
  const createBallotPendingPrintMutation =
    createBallotPendingPrint.useMutation();

  const submitBallot = useCallback(
    async (pin: string) => {
      setError(undefined);
      setIsCheckingPin(true);
      try {
        const ballotPendingPrintId =
          await createBallotPendingPrintMutation.mutateAsync(
            { votes, pin },
            {
              onError(err) {
                setError(err instanceof Error ? err.message : `${err}`);
              },
            }
          );
        onSubmitted(ballotPendingPrintId);
      } catch (err) {
        setError(err instanceof Error ? err.message : `${err}`);
      } finally {
        setIsCheckingPin(false);
      }
    },
    [createBallotPendingPrintMutation, onSubmitted, votes]
  );

  const handleNumberEntry = useCallback(
    async (digit: number) => {
      const pin = pinEntry.handleDigit(digit);
      if (pin.length === pinLength.max) {
        await submitBallot(pin);
      }
    },
    [pinEntry, submitBallot]
  );

  const handleEnter = useCallback(async () => {
    await submitBallot(pinEntry.current);
  }, [pinEntry, submitBallot]);

  return (
    <Screen white>
      <Main centerChild>
        <Prose
          textCenter
          themeDeprecated={fontSizeTheme.medium}
          maxWidth={false}
        >
          <P>Enter the card PIN to digitally sign your ballot.</P>
          {error && <P>{error}</P>}
          <EnteredCode>{pinEntry.display}</EnteredCode>
          <NumberPadWrapper>
            <NumberPad
              disabled={isCheckingPin}
              onButtonPress={handleNumberEntry}
              onBackspace={pinEntry.handleBackspace}
              onClear={pinEntry.reset}
              onEnter={handleEnter}
            />
            {!pinLength.isFixed && (
              <Button onPress={handleEnter} disabled={isCheckingPin}>
                Enter
              </Button>
            )}
          </NumberPadWrapper>
        </Prose>
      </Main>
    </Screen>
  );
}
