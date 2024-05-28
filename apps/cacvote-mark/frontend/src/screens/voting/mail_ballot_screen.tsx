import { Button, Main, P, Screen } from '@votingworks/ui';
import styled from 'styled-components';
import { MailBallotIcon } from '../../components/mail_ballot_icon';
import {
  Wizard,
  WizardButtonBar,
  WizardHeaderTitle,
} from '../../components/wizard';
import { WizardSteps } from '../../components/wizard_step';
import { steps } from './steps';

const Instructions = styled(P)`
  font-weight: 300;
  margin: 0 auto 100px auto;
  max-width: 50%;
  text-align: center;
`;

export interface MailBallotScreenProps {
  onBack: () => void;
  onDone: () => void;
}

export function MailBallotScreen({
  onBack,
  onDone,
}: MailBallotScreenProps): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <Wizard
          header={<WizardHeaderTitle step="Step 4" title="Mail Ballot" />}
          footer={<WizardSteps steps={steps} current="mail" />}
          actions={
            <WizardButtonBar
              leftButton={
                <Button icon="Previous" onPress={onBack}>
                  Step 3: Attach Mail Label
                </Button>
              }
              rightButton={
                <Button onPress={onDone} variant="primary">
                  Done
                </Button>
              }
            />
          }
        >
          <Instructions>
            To cast your vote, drop your envelope in the mail box.
          </Instructions>
          <MailBallotIcon />
        </Wizard>
      </Main>
    </Screen>
  );
}
