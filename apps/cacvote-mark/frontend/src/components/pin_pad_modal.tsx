import {
  Button,
  Modal,
  NumberPad,
  PinLength,
  Text,
  usePinEntry,
} from '@votingworks/ui';
import React from 'react';
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
  pinLength: PinLength;
  primaryButtonLabel: string;
  dismissButtonLabel: string;
  onEnter: (pin: string) => void;
  onDismiss: () => void;
  title?: string;
  disabled?: boolean;
  error?: string;
}

export function PinPadModal({
  pinLength,
  primaryButtonLabel,
  dismissButtonLabel,
  onEnter,
  onDismiss,
  title = 'Enter Your PIN',
  disabled,
  error,
}: PinPadModalProps): JSX.Element {
  const pinEntry = usePinEntry({ pinLength });

  function handleEnter() {
    onEnter(pinEntry.current);
  }

  function dismissPinModal() {
    pinEntry.reset();
    onDismiss();
  }

  return (
    <Modal
      title={title}
      ariaLabel="Enter your PIN"
      onOverlayClick={dismissPinModal}
      content={
        <React.Fragment>
          {error && <Text error>Error: {error}</Text>}
          <EnteredCode>{pinEntry.display}</EnteredCode>
          <NumberPadWrapper>
            <NumberPad
              disabled={disabled}
              onButtonPress={pinEntry.handleDigit}
              onBackspace={pinEntry.handleBackspace}
              onClear={pinEntry.reset}
              onEnter={handleEnter}
            />
          </NumberPadWrapper>
        </React.Fragment>
      }
      actions={
        <React.Fragment>
          <Button
            variant="primary"
            onPress={handleEnter}
            disabled={disabled}
            aria-label="enter"
          >
            {primaryButtonLabel}
          </Button>
          <Button onPress={dismissPinModal} aria-label="cancel">
            {dismissButtonLabel}
          </Button>
        </React.Fragment>
      }
    />
  );
}
