import { assert, assertDefined } from '@votingworks/basics';
import {
  ElectionDefinition,
  safeParseElectionDefinition,
} from '@votingworks/types';
import { Button, FileInputButton, InputGroup, Modal, P } from '@votingworks/ui';
import React, { useState } from 'react';
import styled from 'styled-components';

const Label = styled.div`
  display: block;
  font-weight: ${(p) => p.theme.sizes.fontWeight.semiBold};
`;

const ErrorMessage = styled(P)`
  color: ${(p) => p.theme.colors.danger};
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
  errorMessage?: string;
}

export function CreateElectionModal({
  onCreate,
  onClose,
  errorMessage,
}: CreateElectionModalProps): JSX.Element {
  const [mailingAddress, setMailingAddress] = useState('');
  const [electionDefinition, setElectionDefinition] =
    useState<ElectionDefinition>();
  const isReadyToCreate =
    mailingAddress.length > 0 && electionDefinition !== undefined;

  function onPressCreate() {
    assert(electionDefinition);
    assert(mailingAddress.length > 0);
    onCreate({ mailingAddress, electionDefinition });
  }

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
          <InputGroup>
            <Label>Mailing Address</Label>
            <textarea
              placeholder="Where should ballots be sent?"
              rows={4}
              value={mailingAddress}
              onInput={(event) => setMailingAddress(event.currentTarget.value)}
            />
          </InputGroup>
          <InputGroup>
            <Label>Election Definition</Label>
            <FileInputButton
              accept=".json"
              onChange={(event) => {
                const input = event.currentTarget;
                const { files } = input;
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
              Import Election
            </FileInputButton>
          </InputGroup>
          {errorMessage && <ErrorMessage>{errorMessage}</ErrorMessage>}
        </React.Fragment>
      }
      actions={
        <React.Fragment>
          <Button
            variant="primary"
            onPress={onPressCreate}
            disabled={!isReadyToCreate}
          >
            Create
          </Button>
          <Button variant="neutral" onPress={onClose}>
            Cancel
          </Button>
        </React.Fragment>
      }
    />
  );
}
