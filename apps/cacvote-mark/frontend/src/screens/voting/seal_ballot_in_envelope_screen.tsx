import { Button, H3, H5, Main, Screen } from '@votingworks/ui';
import styled from 'styled-components';
import { Wizard, WizardButtonBar } from '../../components/wizard';
import { WizardSteps } from '../../components/wizard_step';
import { steps } from './steps';
import {
  IconWithInstructions,
  InstructionSection,
} from '../../components/instructions';
import { InsertBallotInEnvelopeIcon } from '../../components/insert_ballot_in_envelope_icon';
import { RemoveAdhesiveIcon } from '../../components/remove_adhesive_icon';
import { PressDownFlapIcon } from '../../components/press_down_flap_icon';
import { ConfirmSealIcon } from '../../components/confirm_seal_icon';

const InstructionsContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;

  > * {
    height: 160px;
  }
`;

const Header = styled.div`
  padding: 60px 20px 0 20px;
`;

export interface SealBallotInEnvelopeScreenProps {
  onNext: () => void;
}

export function SealBallotInEnvelopeScreen({
  onNext,
}: SealBallotInEnvelopeScreenProps): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <Wizard
          header={
            <Header>
              <H5>Step 2</H5>
              <H3>Seal Ballot in Envelope</H3>
            </Header>
          }
          footer={<WizardSteps steps={steps} current="seal" />}
          actions={
            <WizardButtonBar
              rightButton={
                <Button rightIcon="Next" onPress={onNext} variant="primary">
                  Step 3: Print Mail Label
                </Button>
              }
            />
          }
        >
          <InstructionsContainer>
            <InstructionSection padded={false}>
              <IconWithInstructions icon={<InsertBallotInEnvelopeIcon />}>
                Insert ballot in envelope.
              </IconWithInstructions>
            </InstructionSection>
            <InstructionSection padded={false}>
              <IconWithInstructions icon={<RemoveAdhesiveIcon />}>
                Peel off adhesive tape.
              </IconWithInstructions>
            </InstructionSection>
            <InstructionSection padded={false}>
              <IconWithInstructions icon={<PressDownFlapIcon />}>
                Firmly fold down flap.
              </IconWithInstructions>
            </InstructionSection>
            <InstructionSection padded={false}>
              <IconWithInstructions icon={<ConfirmSealIcon />}>
                Make sure it&rsquo;s sealed tight.
              </IconWithInstructions>
            </InstructionSection>
          </InstructionsContainer>
        </Wizard>
      </Main>
    </Screen>
  );
}
