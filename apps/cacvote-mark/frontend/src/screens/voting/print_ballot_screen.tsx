import { useBallotPrinter } from '@votingworks/mark-flow-ui';
import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import { H2, Main, P, Screen, appStrings, printElement } from '@votingworks/ui';
import { useEffect, useRef } from 'react';
import styled from 'styled-components';
import { ArrowDownIcon } from '../../components/arrow_down_icon';
import { BallotIcon } from '../../components/ballot_icon';
import { ThermalPrinterIcon } from '../../components/thermal_printer_icon';

export interface PrintBallotScreenProps {
  electionDefinition: ElectionDefinition;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
  votes: VotesDict;
  generateBallotId: () => string;
  isLiveMode: boolean;
  onPrintCompleted: () => void;
}

const ArrowDownIconContainer = styled.div`
  margin-top: 40px;
`;

export function PrintBallotScreenStatic(): JSX.Element {
  return (
    <Screen>
      <Main centerChild padded>
        <H2>{appStrings.titleBmdPrintScreen()}</H2>
        <P
          style={{
            marginTop: '20px',
            marginBottom: '40px',
            textAlign: 'center',
          }}
        >
          Tear off ballot from printer
          <br />
          and come back for instructions.
        </P>
        <ThermalPrinterIcon />
        <BallotIcon />
        <ArrowDownIconContainer>
          <ArrowDownIcon />
        </ArrowDownIconContainer>
      </Main>
    </Screen>
  );
}

export function PrintBallotScreen({
  onPrintCompleted,
  ...rest
}: PrintBallotScreenProps): JSX.Element {
  const printTimer = useRef<number>();

  const printBallot = useBallotPrinter({
    ...rest,

    printElement: (element, printOptions) =>
      printElement(element, {
        ...printOptions,
        deviceName: process.env.REACT_APP_BALLOT_PRINTER,
      }),

    onPrintStarted() {
      printTimer.current = window.setTimeout(
        onPrintCompleted,
        process.env.IS_INTEGRATION_TEST === 'true' ? 500 : 5000
      );
    },
  });

  useEffect(() => {
    printBallot();
    return () => {
      window.clearTimeout(printTimer.current);
    };
  }, [printBallot]);

  return <PrintBallotScreenStatic />;
}
