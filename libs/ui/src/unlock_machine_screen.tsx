import { useState, useCallback } from 'react';
import styled from 'styled-components';
import { DippedSmartCardAuth, InsertedSmartCardAuth } from '@votingworks/types';
import { assert } from '@votingworks/basics';

import { Screen } from './screen';
import { Main } from './main';
import { Prose } from './prose';
import { fontSizeTheme } from './themes';
import { NumberPad } from './number_pad';
import { useNow } from './hooks/use_now';
import { Timer } from './timer';
import { P } from './typography';
import { Icons } from './icons';
import { Button } from './button';
import { PinLength } from './utils/pin_length';
import { usePinEntry } from './hooks/use_pin_entry';

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

type CheckingPinAuth =
  | DippedSmartCardAuth.CheckingPin
  | InsertedSmartCardAuth.CheckingPin;

interface Props {
  auth: CheckingPinAuth;
  checkPin: (pin: string) => Promise<void>;
  pinLength: PinLength;
  grayBackground?: boolean;
}

export function UnlockMachineScreen({
  auth,
  checkPin,
  pinLength,
  grayBackground,
}: Props): JSX.Element {
  const pinEntry = usePinEntry({ pinLength });
  const [isCheckingPin, setIsCheckingPin] = useState(false);
  const now = useNow().toJSDate();

  const submitPin = useCallback(
    async (pin: string) => {
      setIsCheckingPin(true);
      await checkPin(pin);
      pinEntry.reset();
      setIsCheckingPin(false);
    },
    [checkPin, pinEntry]
  );

  const handleNumberEntry = useCallback(
    async (digit: number) => {
      const pin = pinEntry.handleDigit(digit);
      if (pin.length === pinLength.max) {
        await submitPin(pin);
      }
    },
    [pinEntry, pinLength.max, submitPin]
  );

  const handleEnter = useCallback(async () => {
    await submitPin(pinEntry.current);
  }, [pinEntry, submitPin]);

  const isLockedOut = Boolean(
    auth.lockedOutUntil && now < new Date(auth.lockedOutUntil)
  );
  const pinEntryDisabled = isCheckingPin || isLockedOut;

  let primarySentence: JSX.Element = <P>Enter the card PIN to unlock.</P>;
  if (auth.error) {
    primarySentence = (
      <P color="danger">
        <Icons.Danger /> Error checking PIN. Please try again.
      </P>
    );
  } else if (isLockedOut) {
    assert(auth.lockedOutUntil !== undefined);
    primarySentence = (
      <P color="warning">
        <Icons.Warning /> Card locked. Please try again in{' '}
        <Timer countDownTo={new Date(auth.lockedOutUntil)} />
      </P>
    );
  } else if (auth.wrongPinEnteredAt) {
    primarySentence = (
      <P color="warning">
        <Icons.Warning /> Incorrect PIN. Please try again.
      </P>
    );
  }

  return (
    <Screen white={!grayBackground}>
      <Main centerChild>
        <Prose
          textCenter
          themeDeprecated={fontSizeTheme.medium}
          maxWidth={false}
        >
          {primarySentence}
          <EnteredCode>{pinEntry.display}</EnteredCode>
          <NumberPadWrapper>
            <NumberPad
              disabled={pinEntryDisabled}
              onButtonPress={handleNumberEntry}
              onBackspace={pinEntry.handleBackspace}
              onEnter={handleEnter}
              onClear={pinEntry.reset}
            />
            {!pinLength.isFixed && (
              <Button onPress={handleEnter} disabled={pinEntryDisabled}>
                Enter
              </Button>
            )}
          </NumberPadWrapper>
        </Prose>
      </Main>
    </Screen>
  );
}
