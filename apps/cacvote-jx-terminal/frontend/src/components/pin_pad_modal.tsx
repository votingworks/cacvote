import {
  Modal,
  NumberPad,
  PinLength,
  Text,
  usePinEntry,
} from '@votingworks/ui';
import React, { useEffect } from 'react';
import styled from 'styled-components';

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

export interface PinPadModalProps {
  onEnter: (pin: string) => void;
  title?: string;
  isAuthenticating?: boolean;
  error?: string;
}

export function PinPadModal({
  onEnter,
  title = 'Enter Your PIN',
  isAuthenticating,
  error,
}: PinPadModalProps): JSX.Element {
  const pinLength = PinLength.exactly(6);
  const pinEntry = usePinEntry({ pinLength });

  useEffect(() => {
    if (pinEntry.current.length === pinLength.max) {
      onEnter(pinEntry.current);
      pinEntry.reset();
    }
  }, [onEnter, pinEntry, pinLength.max]);

  return (
    <Modal
      title={title}
      ariaLabel="Enter your PIN"
      content={
        <React.Fragment>
          {error && <Text error>Error: {error}</Text>}
          <EnteredCode>{pinEntry.display}</EnteredCode>
          <NumberPadWrapper>
            <NumberPad
              disabled={isAuthenticating}
              onButtonPress={pinEntry.handleDigit}
              onBackspace={pinEntry.handleBackspace}
              onClear={pinEntry.reset}
            />
          </NumberPadWrapper>
        </React.Fragment>
      }
    />
  );
}
