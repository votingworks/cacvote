import { useEffect, useRef } from 'react';
import { PrintPage as MarkFlowPrintPage } from '@votingworks/mark-flow-ui';
import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import { saveVotes } from '../../api';

export interface PrintScreenProps {
  electionDefinition: ElectionDefinition;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
  votes: VotesDict;
  generateBallotId: () => string;
  isLiveMode: boolean;
}

export function PrintScreen({
  electionDefinition,
  ballotStyleId,
  precinctId,
  votes,
  generateBallotId,
  isLiveMode,
}: PrintScreenProps): JSX.Element {
  const printTimer = useRef<number>();
  const saveVotesMutation = saveVotes.useMutation();

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
          () => {
            saveVotesMutation.mutate({ votes });
          },
          process.env.IS_INTEGRATION_TEST === 'true' ? 500 : 5000
        );
      }}
    />
  );
}
