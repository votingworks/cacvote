import { Button, Main, Screen } from '@votingworks/ui';
import {
  Wizard,
  WizardButtonBar,
  WizardHeaderTitle,
} from '../../components/wizard';
import { WizardSteps } from '../../components/wizard_step';
import { steps } from './steps';
import { RemoveCommonAccessCardIcon } from '../../components/remove_common_access_card_icon';

export interface RemoveCommonAccessCardToPrintMailLabelScreenProps {
  onCancel: () => void;
}

export function RemoveCommonAccessCardToPrintMailLabelScreen({
  onCancel,
}: RemoveCommonAccessCardToPrintMailLabelScreenProps): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <Wizard
          header={
            <WizardHeaderTitle
              step="Step 3"
              title="Remove Common Access Card to Print Mail Label"
            />
          }
          footer={<WizardSteps steps={steps} current="label" />}
          actions={
            <WizardButtonBar
              leftButton={
                <Button icon="Previous" onPress={onCancel}>
                  Step 2: Seal Ballot in Envelope
                </Button>
              }
            />
          }
        >
          <RemoveCommonAccessCardIcon />
        </Wizard>
      </Main>
    </Screen>
  );
}
