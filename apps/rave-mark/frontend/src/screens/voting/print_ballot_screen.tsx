import { PrintPage as MarkFlowPrintPage } from '@votingworks/mark-flow-ui';
import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  PrintOptions,
  VotesDict,
} from '@votingworks/types';
import { Button, H1, Main, P, Screen, printElement } from '@votingworks/ui';
import { useEffect, useRef, useState } from 'react';

export interface PrintBallotScreenProps {
  electionDefinition: ElectionDefinition;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
  votes: VotesDict;
  generateBallotId: () => string;
  isLiveMode: boolean;
  onPrintCompleted: () => void;
}

export function PrintBallotScreen({
  electionDefinition,
  ballotStyleId,
  precinctId,
  votes,
  generateBallotId,
  isLiveMode,
  onPrintCompleted,
}: PrintBallotScreenProps): JSX.Element {
  const printTimer = useRef<number>();
  const [hasInsertedBallotPaper, setHasInsertedBallotPaper] = useState(false);

  useEffect(() => {
    return () => {
      window.clearTimeout(printTimer.current);
    };
  }, []);

  function printElementToBallotPrinter(
    element: JSX.Element,
    printOptions: PrintOptions
  ) {
    return printElement(element, {
      ...printOptions,
      deviceName: process.env.REACT_APP_BALLOT_PRINTER,
    });
  }

  if (!hasInsertedBallotPaper) {
    return (
      <Screen>
        <Main centerChild>
          <H1>Insert Blank Ballot Sheet</H1>
          <P>Insert a blank sheet of thermal ballot paper into the printer.</P>
          <P>
            After inserting the blank sheet, press the button below to print
            your ballot.
          </P>
          <Button
            variant="primary"
            onPress={() => setHasInsertedBallotPaper(true)}
          >
            Sheet Inserted — Print Ballot
          </Button>
        </Main>
      </Screen>
    );
  }

  return (
    <MarkFlowPrintPage
      electionDefinition={electionDefinition}
      ballotStyleId={ballotStyleId}
      precinctId={precinctId}
      generateBallotId={generateBallotId}
      isLiveMode={isLiveMode}
      votes={votes}
      printElement={printElementToBallotPrinter}
      onPrintStarted={() => {
        printTimer.current = window.setTimeout(
          onPrintCompleted,
          process.env.IS_INTEGRATION_TEST === 'true' ? 500 : 5000
        );
      }}
    />
  );
}
