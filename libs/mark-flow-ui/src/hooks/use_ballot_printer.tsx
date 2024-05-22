import { useCallback } from 'react';

import {
  BmdPaperBallot,
  printElement as DefaultPrintElement,
  useLock,
} from '@votingworks/ui';

import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  PrintOptions,
  VotesDict,
} from '@votingworks/types';

export interface UseBallotPrinterProps {
  electionDefinition: ElectionDefinition;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
  isLiveMode: boolean;
  votes: VotesDict;
  generateBallotId: () => string;
  onPrintStarted?: () => void;
  printElement?: (
    element: JSX.Element,
    printOptions: PrintOptions
  ) => Promise<void>;
  largeTopMargin?: boolean;
}

export function useBallotPrinter({
  electionDefinition,
  ballotStyleId,
  precinctId,
  isLiveMode,
  votes,
  generateBallotId,
  onPrintStarted,
  printElement = DefaultPrintElement,
  largeTopMargin,
}: UseBallotPrinterProps): () => void {
  const printLock = useLock();

  return useCallback(async () => {
    /* istanbul ignore if */
    if (!printLock.lock()) return;
    await printElement(
      <BmdPaperBallot
        ballotStyleId={ballotStyleId}
        electionDefinition={electionDefinition}
        generateBallotId={generateBallotId}
        isLiveMode={isLiveMode}
        precinctId={precinctId}
        votes={votes}
        largeTopMargin={largeTopMargin}
      />,
      { sides: 'one-sided' }
    );
    onPrintStarted?.();
  }, [
    printLock,
    ballotStyleId,
    electionDefinition,
    generateBallotId,
    isLiveMode,
    precinctId,
    votes,
    onPrintStarted,
    printElement,
    largeTopMargin,
  ]);
}
