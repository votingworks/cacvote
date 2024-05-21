import { throwIllegalValue } from '@votingworks/basics';
import { Button, Main, P, Screen } from '@votingworks/ui';
import { useState } from 'react';
import styled from 'styled-components';
import { RadioBox } from '../../components/radio_box';
import {
  Wizard,
  WizardButtonBar,
  WizardHeaderTitle,
} from '../../components/wizard';
import { WizardSteps } from '../../components/wizard_step';

const ButtonsRow = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-evenly;
  align-items: center;
  width: 100%;
  margin-top: 20px;
`;

export interface ReviewPrintedBallotScreenProps {
  onConfirm: () => void;
  onReject: () => void;
}

enum ReviewStepChoice {
  Yes = 'yes',
  No = 'no',
}

const FirstWord = styled.div`
  font-weight: 700;
`;

export function ReviewPrintedBallotScreen({
  onConfirm: onConfirmPrintedBallotSelections,
  onReject: onRejectPrintedBallotSelections,
}: ReviewPrintedBallotScreenProps): JSX.Element {
  const [choice, setChoice] = useState<ReviewStepChoice>();

  function onNext() {
    if (choice === ReviewStepChoice.Yes) {
      onConfirmPrintedBallotSelections();
    } else if (choice === ReviewStepChoice.No) {
      onRejectPrintedBallotSelections();
    }
  }

  return (
    <Screen>
      <Main centerChild>
        <Wizard
          header={
            <WizardHeaderTitle step="Step 1" title="Review Your Ballot" />
          }
          footer={
            <WizardSteps
              current="review"
              steps={[
                { id: 'review', title: 'Review Ballot' },
                { id: 'seal', title: 'Seal Ballot in Envelope' },
                { id: 'label', title: 'Attach Mail Label' },
                { id: 'mail', title: 'Mail Ballot' },
              ]}
            />
          }
          actions={
            <WizardButtonBar
              rightButton={
                <Button
                  rightIcon={choice ? 'Next' : undefined}
                  onPress={onNext}
                  disabled={!choice}
                  variant={choice ? 'primary' : undefined}
                >
                  {(() => {
                    switch (choice) {
                      case ReviewStepChoice.Yes:
                        return 'Up Next: Enter Pin';
                      case ReviewStepChoice.No:
                        return 'Up Next: Destroy Ballot';
                      case undefined:
                        return 'Make a selection';
                      default:
                        throwIllegalValue(choice);
                    }
                  })()}
                </Button>
              }
            />
          }
        >
          <P style={{ fontSize: '34px', fontWeight: 400 }}>
            Are all the printed selections correct?
          </P>
          <ButtonsRow>
            <RadioBox
              selected={choice === ReviewStepChoice.Yes}
              onClick={() => setChoice(ReviewStepChoice.Yes)}
            >
              <FirstWord>Yes,</FirstWord> My printed selections are correct.
            </RadioBox>
            <RadioBox
              selected={choice === ReviewStepChoice.No}
              onClick={() => setChoice(ReviewStepChoice.No)}
            >
              <FirstWord>No,</FirstWord> I need to make changes.
            </RadioBox>
          </ButtonsRow>
        </Wizard>
      </Main>
    </Screen>
  );
}
