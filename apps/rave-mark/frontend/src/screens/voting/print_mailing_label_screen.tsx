import {
  H1,
  Main,
  PrintingBallotImage,
  Prose,
  Screen,
  printElement,
  useLock,
} from '@votingworks/ui';
import { useEffect, useRef } from 'react';
import { MailingLabel } from '../../components/mailing_label';

export interface PrintMailingLabelScreenProps {
  onPrintCompleted: () => void;
}

export function PrintMailingLabelScreen({
  onPrintCompleted,
}: PrintMailingLabelScreenProps): JSX.Element {
  const printLock = useLock();
  const printTimer = useRef<number>();

  useEffect(() => {
    if (!printLock.lock()) return;

    printTimer.current = window.setTimeout(
      onPrintCompleted,
      process.env.IS_INTEGRATION_TEST === 'true' ? 100 : 2000
    );

    void printElement(<MailingLabel />, {
      sides: 'one-sided',
      deviceName: process.env.REACT_APP_MAILING_LABEL_PRINTER_NAME,
      raw: { media: 'Custom.4x6in' },
    });

    return () => {
      printLock.unlock();
      window.clearTimeout(printTimer.current);
    };
  }, [onPrintCompleted, printLock]);

  return (
    <Screen white>
      <Main centerChild padded>
        <Prose textCenter id="audiofocus">
          <PrintingBallotImage />
          <H1>Printing Your Mailing Label…</H1>
        </Prose>
      </Main>
    </Screen>
  );
}
