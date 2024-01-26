import { H2, Main, P, Screen, Text } from '@votingworks/ui';
import styled from 'styled-components';
import { InsertBallotIntoEnvelopeIcon } from './insert_ballot_into_envelope_icon';
import { Envelope9x12Icon } from './envelope_9x12_icon';
import { EnvelopeFoldFlapIcon } from './envelope_fold_flap_icon';
import { LabelStickerIcon } from './label_sticker_icon';

const InstructionSteps = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: center;
  margin: 20px 10px;
`;

const InstructionStep = styled.div`
  background-color: white;
  border: 1px solid #000;
  border-radius: 10px;
  padding: 10px;
  margin: 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: end;
  text-align: center;
  width: 30%;

  > * + * {
    margin-top: 20px;
  }
`;

export function PostVoteScreen(): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <H2>Mail Your Paper Ballot</H2>
        <InstructionSteps>
          <InstructionStep>
            <InsertBallotIntoEnvelopeIcon />
            <P>Insert your ballot into the envelope.</P>
          </InstructionStep>
          <InstructionStep>
            <Envelope9x12Icon />
            <EnvelopeFoldFlapIcon />
            <P>Peel off the adhesive tape and press down on the flap.</P>
          </InstructionStep>
          <InstructionStep>
            <LabelStickerIcon />
            <P>
              Peel off label sticker.
              <br />
              Stick it in the center on the back of the envelope.
            </P>
          </InstructionStep>
        </InstructionSteps>
        <Text center small>
          Remove your card when you’re ready to end your voting session.
        </Text>
      </Main>
    </Screen>
  );
}
