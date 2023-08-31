import { VotesDict } from '@votingworks/types';
import {
  Button,
  H1,
  Main,
  Modal,
  NumberPad,
  P,
  Prose,
  Screen,
  fontSizeTheme,
  usePinEntry,
} from '@votingworks/ui';
import React, { useState } from 'react';
import styled from 'styled-components';
import { castBallot } from '../../api';
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
  onSubmitted: () => void;
}

const pinLength = COMMON_ACCESS_CARD_PIN_LENGTH;

export function SubmitScreen({
  votes,
  onSubmitted,
}: SubmitScreenProps): JSX.Element {
  const [error, setError] = useState<string>();
  const [isShowingPinModal, setIsShowingPinModal] = useState(false);
  const [isCheckingPin, setIsCheckingPin] = useState(false);
  const pinEntry = usePinEntry({ pinLength });
  const castBallotMutation = castBallot.useMutation();
  const castBallotMutationMutateAsync = castBallotMutation.mutateAsync;

  async function submitBallot(pin: string) {
    setError(undefined);
    setIsCheckingPin(true);
    try {
      await castBallotMutationMutateAsync(
        { votes, pin },
        {
          onError(err) {
            setError(err instanceof Error ? err.message : `${err}`);
          },
        }
      );
      onSubmitted();
    } catch (err) {
      setError(err instanceof Error ? err.message : `${err}`);
    } finally {
      setIsCheckingPin(false);
    }
  }

  async function handleEnter() {
    await submitBallot(pinEntry.current);
  }

  function dismissPinModal() {
    setIsShowingPinModal(false);
    pinEntry.reset();
  }

  return (
    <Screen white>
      <Main centerChild>
        <Prose
          textCenter
          themeDeprecated={fontSizeTheme.medium}
          maxWidth={false}
        >
          <H1>Cast My Ballot</H1>
          <P>
            Thanks for reviewing your printed ballot!
            <br />
            Now, we&rsquo;ll sign and cast your electronic ballot.
          </P>
          <Button variant="primary" onPress={() => setIsShowingPinModal(true)}>
            Sign &amp; Cast My Ballot
          </Button>
          {isShowingPinModal && (
            <Modal
              title="Enter Your PIN"
              onOverlayClick={dismissPinModal}
              content={
                <React.Fragment>
                  {error && <P>{error}</P>}
                  <EnteredCode>{pinEntry.display}</EnteredCode>
                  <NumberPadWrapper>
                    <NumberPad
                      disabled={isCheckingPin}
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
                    disabled={isCheckingPin}
                  >
                    {isCheckingPin ? 'Checking…' : 'Submit'}
                  </Button>
                  <Button onPress={dismissPinModal}>Back</Button>
                </React.Fragment>
              }
            />
          )}
        </Prose>
      </Main>
    </Screen>
  );
}
