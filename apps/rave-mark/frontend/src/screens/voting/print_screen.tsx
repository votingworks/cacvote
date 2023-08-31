import { PrintPage as MarkFlowPrintPage } from '@votingworks/mark-flow-ui';
import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import { useEffect, useRef } from 'react';

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

  useEffect(() => {
    return () => {
      window.clearTimeout(printTimer.current);
    };
  }, []);

  return (
    <MarkFlowPrintPage
      electionDefinition={electionDefinition}
      ballotStyleId={ballotStyleId}
      precinctId={precinctId}
      generateBallotId={generateBallotId}
      isLiveMode={isLiveMode}
      votes={votes}
      onPrintStarted={() => {
        printTimer.current = window.setTimeout(
          onPrintCompleted,
          process.env.IS_INTEGRATION_TEST === 'true' ? 500 : 5000
        );
      }}
    />
  );
}
