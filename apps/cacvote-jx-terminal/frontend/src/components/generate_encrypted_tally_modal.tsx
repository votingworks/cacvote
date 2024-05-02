import { Button, LoadingButton, Modal, P } from '@votingworks/ui';
import React, { useCallback } from 'react';
import { format } from '@votingworks/utils';
import { useModalKeybindings } from '../use_modal_keybindings';

export interface GenerateEncryptedTallyModalProps {
  onGenerate: () => Promise<void>;
  onClose: () => void;
  registeredVoterCount: number;
  castBallotCount: number;
  isGenerating?: boolean;
}

export function GenerateEncryptedTallyModal({
  onGenerate,
  onClose,
  registeredVoterCount,
  castBallotCount,
  isGenerating,
}: GenerateEncryptedTallyModalProps): JSX.Element {
  const onConfirm = useCallback(async () => {
    if (!isGenerating) {
      await onGenerate();
      onClose();
    }
  }, [isGenerating, onClose, onGenerate]);

  const onCancel = useCallback(() => {
    if (!isGenerating) {
      onClose();
    }
  }, [isGenerating, onClose]);

  useModalKeybindings({ onEnter: onConfirm, onEscape: onCancel });

  return (
    <Modal
      title="Generate Encrypted Tally"
      content={
        <React.Fragment>
          <P>
            Please note that this is a <strong>ONE-TIME OPERATION</strong>. Once
            an encrypted tally has been generated, it cannot be undone.
          </P>
          <P>
            You have {format.count(registeredVoterCount)} registered voters and{' '}
            {format.count(castBallotCount)} cast ballots in this election. The
            encrypted tally will include only these cast ballots.
          </P>
          <P>Would you like to proceed?</P>
        </React.Fragment>
      }
      actions={
        <React.Fragment>
          {isGenerating ? (
            <LoadingButton variant="primary">
              Generating Encrypted Tally…
            </LoadingButton>
          ) : (
            <Button
              variant="primary"
              onPress={onConfirm}
              disabled={isGenerating}
            >
              Yes, Generate Encrypted Tally
            </Button>
          )}
          <Button variant="neutral" onPress={onCancel} disabled={isGenerating}>
            Cancel
          </Button>
        </React.Fragment>
      }
    />
  );
}
