import { Button, Main, P, Screen } from '@votingworks/ui';
import styled from 'styled-components';
import {
  Wizard,
  WizardButtonBar,
  WizardHeaderTitle,
} from '../../components/wizard';
import { TornPaperIcon } from '../../components/torn_paper_icon';
import { ThrowAwayIcon } from '../../components/throw_away_icon';

const InstructionSection = styled.div`
  display: flex;
  border: 3px solid #000000;
  width: 90%;
  margin-top: 30px;
  padding: 40px 50px 40px 50px;
`;

const Instructions = styled(P)`
  flex: 1;
  display: flex;
  align-items: center;
  padding-left: 50px;
  font-weight: 400;
`;

const Prompt = styled(P)`
  font-size: 34px;
  font-weight: 400;
  text-align: center;
  padding: 0 60px;
`;

export interface DestroyBallotScreenProps {
  onConfirm: () => void;
  onCancel: () => void;
}

export function DestroyBallotScreen({
  onConfirm,
  onCancel,
}: DestroyBallotScreenProps): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <Wizard
          header={<WizardHeaderTitle title="Destroy Your Ballot" />}
          footer={<div />}
          actions={
            <WizardButtonBar
              leftButton={
                <Button icon="Previous" onPress={onCancel}>
                  Go Back to Step 1
                </Button>
              }
              rightButton={
                <Button variant="primary" onPress={onConfirm}>
                  I Destroyed My Ballot
                </Button>
              }
            />
          }
        >
          <Prompt>
            Since your printed selections were not correct, you must do the
            following:
          </Prompt>
          <InstructionSection>
            <TornPaperIcon />
            <Instructions>
              Rip your paper ballot in small pieces so that no one can read your
              votes.
            </Instructions>
          </InstructionSection>
          <InstructionSection>
            <ThrowAwayIcon />
            <Instructions>Throw away the pieces in a trash can.</Instructions>
          </InstructionSection>
        </Wizard>
      </Main>
    </Screen>
  );
}
