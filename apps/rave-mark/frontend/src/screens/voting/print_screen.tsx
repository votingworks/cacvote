import { useEffect, useRef } from 'react';
import { PrintPage as MarkFlowPrintPage } from '@votingworks/mark-flow-ui';
import {
  BallotStyleId,
  ElectionDefinition,
  Id,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import { sleep } from '@votingworks/basics';
import { markBallotPrinted } from '../../api';

export interface PrintScreenProps {
  electionDefinition: ElectionDefinition;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
  votes: VotesDict;
  generateBallotId: () => string;
  isLiveMode: boolean;
  ballotPendingPrintId: Id;
}

export function PrintScreen({
  electionDefinition,
  ballotStyleId,
  precinctId,
  votes,
  generateBallotId,
  isLiveMode,
  ballotPendingPrintId,
}: PrintScreenProps): JSX.Element {
  const printTimer = useRef<number>();
  const markBallotPrintedMutation = markBallotPrinted.useMutation();

  useEffect(() => {
    return () => {
      window.clearTimeout(printTimer.current);
    };
  }, []);

  async function onPrintStarted() {
    await sleep(process.env.IS_INTEGRATION_TEST === 'true' ? 500 : 5000);
    markBallotPrintedMutation.mutate({ ballotPendingPrintId });
  }

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
          onPrintStarted,
          process.env.IS_INTEGRATION_TEST === 'true' ? 500 : 5000
        );
      }}
    />
  );
}
