import React from 'react';
import { Button, H2, Icons, Main, P, Screen } from '@votingworks/ui';
import { fail, throwIllegalValue } from '@votingworks/basics';
import { ThermalPrinterIcon } from '../../components/thermal_printer_icon';
import { WizardButtonBar } from '../../components/wizard';
import { PrinterStatus } from './print_ballot_screen';

export interface PrintBallotErrorScreenProps {
  status: PrinterStatus;
  onTryAgain: () => void;
  onBack: () => void;
}

export function PrintBallotErrorScreen({
  status,
  onTryAgain,
  onBack,
}: PrintBallotErrorScreenProps): JSX.Element {
  console.log('PrintBallotErrorScreen', status);
  return (
    <Screen>
      <Main centerChild padded>
        <H2>
          <Icons.Warning /> Ballot Printing Failed
        </H2>
        <P
          style={{
            textAlign: 'center',
            margin: '20px 40px 40px 40px',
          }}
        >
          {status.state === 'cover-open' ? (
            <React.Fragment>
              Please close the printer cover and try again.
              <br />
            </React.Fragment>
          ) : status.state === 'no-paper' ? (
            <React.Fragment>
              Please add paper to the printer and try again.
              <br />
            </React.Fragment>
          ) : status.state === 'error' ? (
            <React.Fragment>
              {status.type === 'hardware' ? (
                <React.Fragment>
                  There was a hardware error with the printer.
                  <br />
                  Please try again or ask a poll worker for help.
                </React.Fragment>
              ) : status.type === 'supply-voltage' ? (
                <React.Fragment>
                  The printer is not receiving enough power.
                  <br />
                  Please try again or ask a poll worker for help.
                </React.Fragment>
              ) : status.type === 'receive-data' ? (
                <React.Fragment>
                  The printer is not receiving data.
                  <br />
                  Please try again or ask a poll worker for help.
                </React.Fragment>
              ) : status.type === 'temperature' ? (
                <React.Fragment>
                  The printer is too hot or too cold.
                  <br />
                  Please try again or ask a poll worker for help.
                </React.Fragment>
              ) : status.type === 'disconnected' ? (
                <React.Fragment>
                  The printer is disconnected.
                  <br />
                  Please try again or ask a poll worker for help.
                </React.Fragment>
              ) : (
                throwIllegalValue(status.type)
              )}
            </React.Fragment>
          ) : status.state === 'idle' ? (
            fail('printer status was idle, which is not an error state!')
          ) : (
            throwIllegalValue(status)
          )}
        </P>
        <ThermalPrinterIcon />
        <div style={{ padding: '80px', width: '100%' }}>
          <WizardButtonBar
            leftButton={<Button onPress={onBack}>Back</Button>}
            rightButton={
              <Button onPress={onTryAgain} variant="primary">
                Try Again
              </Button>
            }
          />
        </div>
      </Main>
    </Screen>
  );
}
