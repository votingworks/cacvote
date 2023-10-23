import {
  Button,
  Modal,
  P,
  TouchTextInput,
  US_ENGLISH_ALNUM_KEYMAP,
  VirtualKeyboard,
} from '@votingworks/ui';
import React, { useState } from 'react';
import styled from 'styled-components';

interface Props {
  disabled?: boolean;
}

/**
 * A container for the text input, i.e. the background and border.
 */
export const TextInputContainer = styled.div<Props>`
  display: inline-block;
  border: 1px solid #cccccc;
  background: ${({ disabled = false }) => (disabled ? '#dddddd' : '#ffffff')};
  width: 100%;
  padding: 0.35rem 0.5rem;
  line-height: 1.25;
  height: 2.25em;
`;

/**
 * A small text input label that appears above the text input.
 */
export const TextInputLabel = styled.label`
  display: block;
  font-size: 0.55rem;
  font-weight: 500;
  margin-bottom: 0;
  margin-top: -0.25rem;
  color: #666666;
`;

export interface TextInputProps {
  label: string;
  value?: string;
  disabled?: boolean;
  onChange?: (value: string) => void;
  'data-testid'?: string;
}

/**
 * A text input that opens a virtual keyboard when clicked.
 */
export function TextInput({
  label,
  value: valueFromProps = '',
  disabled,
  onChange,
  'data-testid': testId,
}: TextInputProps): JSX.Element {
  const [value, setValue] = useState(valueFromProps);
  const [isEditing, setIsEditing] = useState(false);

  function onClick() {
    setIsEditing(true);
  }

  function onKeyboardBackspace() {
    setValue((prev) => prev.slice(0, -1));
  }

  function onKeyboardKeyPress(key: string) {
    setValue((prev) => prev + key);
  }

  function onAccept() {
    onChange?.(value);
    setIsEditing(false);
  }

  function onCancel() {
    setValue(valueFromProps);
    setIsEditing(false);
  }

  function keyDisabled() {
    return false;
  }

  return (
    <React.Fragment>
      {isEditing && (
        <Modal
          title={label}
          content={
            <React.Fragment>
              <TouchTextInput value={value} />
              <P>&nbsp;</P>
              <VirtualKeyboard
                keyMap={US_ENGLISH_ALNUM_KEYMAP}
                onBackspace={onKeyboardBackspace}
                onKeyPress={onKeyboardKeyPress}
                keyDisabled={keyDisabled}
              />
            </React.Fragment>
          }
          actions={
            <React.Fragment>
              <Button variant="done" onPress={onAccept}>
                Accept
              </Button>
              <Button onPress={onCancel}>Cancel</Button>
            </React.Fragment>
          }
        />
      )}
      <TextInputContainer
        disabled={disabled}
        onClick={onClick}
        data-testid={testId}
      >
        <TextInputLabel>{label}</TextInputLabel>
        {value}
      </TextInputContainer>
    </React.Fragment>
  );
}

export const InlineForm = styled.div`
  display: flex;
  flex-direction: row;
  input[type='text'] {
    flex: 1;
    border-radius: 0.25em;
  }
  & > button:not(:last-child),
  & > input[type='text']:not(:last-child) {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
  }
  & > button:not(:first-child),
  & > input[type='text']:not(:first-child) {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
  }
`;
