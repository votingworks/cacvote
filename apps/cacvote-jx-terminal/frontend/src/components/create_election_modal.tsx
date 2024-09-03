import { assert, assertDefined } from '@votingworks/basics';
import {
  ElectionDefinition,
  safeParseElectionDefinition,
} from '@votingworks/types';
import { Button, FileInputButton, Modal, P } from '@votingworks/ui';
import React, { useCallback, useState } from 'react';
import styled from 'styled-components';
import { useModalKeybindings } from '../use_modal_keybindings';

const Label = styled.label`
  display: block;
  font-weight: ${(p) => p.theme.sizes.fontWeight.semiBold};
`;

export interface CreateElectionModalProps {
  onCreate: ({
    mailingAddress,
    electionDefinition,
  }: {
    mailingAddress: string;
    electionDefinition: ElectionDefinition;
  }) => void;
  onClose: () => void;
  isCreating?: boolean;
}

export function CreateElectionModal({
  onCreate,
  onClose,
  isCreating,
}: CreateElectionModalProps): JSX.Element {
  const [mailingAddress, setMailingAddress] = useState('');
  const [electionDefinition, setElectionDefinition] =
    useState<ElectionDefinition>();
  const isReadyToCreate =
    !isCreating &&
    mailingAddress.length > 0 &&
    electionDefinition !== undefined;

  const onPressCreate = useCallback(() => {
    assert(electionDefinition);
    assert(mailingAddress.length > 0);
    onCreate({ mailingAddress, electionDefinition });
  }, [electionDefinition, mailingAddress, onCreate]);

  const onKeyboardEnter = useCallback(() => {
    if (isReadyToCreate) {
      onPressCreate();
    }
  }, [isReadyToCreate, onPressCreate]);

  useModalKeybindings({ onEnter: onKeyboardEnter, onEscape: onClose });

  return (
    <Modal
      title="Create New Election"
      content={
        <React.Fragment>
          <P>
            To create a new election, you will need to upload an election
            definition file. This file should be in the VotingWorks or CDF
            format.
          </P>
          <Label>
            Mailing Address
            <br />
            <textarea
              placeholder="Where should ballots be sent?"
              rows={4}
              value={mailingAddress}
              onInput={(event) => setMailingAddress(event.currentTarget.value)}
            />
          </Label>
          <Label>
            Election Definition
            <br />
            <FileInputButton
              accept=".json"
              onChange={(event) => {
                const input = event.currentTarget;
                const { files } = input;
                /* istanbul ignore next */
                const file = assertDefined(files?.[0]);
                const reader = new FileReader();
                reader.onload = () => {
                  setElectionDefinition(
                    safeParseElectionDefinition(
                      reader.result as string
                    ).unsafeUnwrap()
                  );
                };
                reader.readAsText(file);
              }}
            >
              {electionDefinition
                ? electionDefinition.election.title
                : 'Choose Election'}
            </FileInputButton>
          </Label>
        </React.Fragment>
      }
      actions={
        <React.Fragment>
          <Button
            variant="primary"
            onPress={onPressCreate}
            disabled={!isReadyToCreate}
          >
            Create Election
          </Button>
          <Button variant="neutral" onPress={onClose}>
            Cancel
          </Button>
        </React.Fragment>
      }
    />
  );
}
