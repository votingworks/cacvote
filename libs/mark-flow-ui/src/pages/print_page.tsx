import {
  printElement as DefaultPrintElement,
  Font,
  H1,
  Main,
  PrintingBallotImage,
  ReadOnLoad,
  Screen,
  appStrings,
} from '@votingworks/ui';
import { useEffect } from 'react';
import {
  UseBallotPrinterProps,
  useBallotPrinter,
} from '../hooks/use_ballot_printer';

export function PrintPageStatic(): JSX.Element {
  return (
    <Screen>
      <Main centerChild padded>
        <Font align="center">
          <PrintingBallotImage />
          <ReadOnLoad>
            <H1>{appStrings.titleBmdPrintScreen()}</H1>
          </ReadOnLoad>
        </Font>
      </Main>
    </Screen>
  );
}

export type PrintPageProps = UseBallotPrinterProps;

export function PrintPage({
  electionDefinition,
  ballotStyleId,
  precinctId,
  isLiveMode,
  votes,
  generateBallotId,
  onPrintStarted,
  printElement = DefaultPrintElement,
  largeTopMargin,
}: PrintPageProps): JSX.Element {
  const printBallot = useBallotPrinter({
    electionDefinition,
    ballotStyleId,
    precinctId,
    isLiveMode,
    votes,
    generateBallotId,
    onPrintStarted,
    printElement,
    largeTopMargin,
  });

  useEffect(() => {
    printBallot();
  }, [printBallot]);

  return <PrintPageStatic />;
}
