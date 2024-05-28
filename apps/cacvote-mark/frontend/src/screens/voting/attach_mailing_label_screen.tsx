import { Button, Main, Screen } from '@votingworks/ui';
import {
  Wizard,
  WizardButtonBar,
  WizardHeaderTitle,
} from '../../components/wizard';
import { WizardSteps } from '../../components/wizard_step';
import { steps } from './steps';
import { AttachMailLabelIcon } from '../../components/attach_mail_label_icon';

export interface AttachMailingLabelScreenProps {
  onReprintMailingLabelPressed: () => void;
  onConfirmMailingLabelAttached: () => void;
}

export function AttachMailingLabelScreen({
  onReprintMailingLabelPressed,
  onConfirmMailingLabelAttached,
}: AttachMailingLabelScreenProps): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <Wizard
          centerContent
          header={
            <WizardHeaderTitle
              step="Step 3"
              title="Attach Mail Label to Envelope"
            />
          }
          footer={<WizardSteps steps={steps} current="label" />}
          actions={
            <WizardButtonBar
              leftButton={
                <Button icon="Warning" onPress={onReprintMailingLabelPressed}>
                  Reprint Mail Label
                </Button>
              }
              rightButton={
                <Button
                  rightIcon="Next"
                  onPress={onConfirmMailingLabelAttached}
                  variant="primary"
                >
                  Step 4: Mail Ballot
                </Button>
              }
            />
          }
        >
          <AttachMailLabelIcon />
        </Wizard>
      </Main>
    </Screen>
  );
}
