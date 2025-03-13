import { Buffer } from 'buffer';
import { useBallotPrinter } from '@votingworks/mark-flow-ui';
import {
  BallotStyleId,
  ElectionDefinition,
  PrecinctId,
  VotesDict,
} from '@votingworks/types';
import {
  H2,
  Main,
  P,
  Screen,
  appStrings,
  printElementToPdf,
} from '@votingworks/ui';
import { useEffect } from 'react';
import styled from 'styled-components';
import { ArrowDownIcon } from '../../components/arrow_down_icon';
import { BallotIcon } from '../../components/ballot_icon';
import { ThermalPrinterIcon } from '../../components/thermal_printer_icon';
import { printBallotPdf } from '../../api';

// Copied from libs/fujitsu-thermal-printer/src/types.ts
export type ErrorType =
  | 'hardware'
  | 'supply-voltage'
  | 'receive-data'
  | 'temperature'
  | 'disconnected';
export type PrinterStatus =
  | {
      state: 'cover-open';
    }
  | {
      state: 'no-paper';
    }
  | {
      state: 'idle';
    }
  | {
      state: 'error';
      type: ErrorType;
    };

export interface PrintBallotScreenProps {
  electionDefinition: ElectionDefinition;
  ballotStyleId: BallotStyleId;
  precinctId: PrecinctId;
  votes: VotesDict;
  generateBallotId: () => string;
  isLiveMode: boolean;
  onPrintCompleted: () => void;
  onPrintError: (printerStatus: PrinterStatus) => void;
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
  onPrintError,
  ...rest
}: PrintBallotScreenProps): JSX.Element {
  const printBallotPdfMutation = printBallotPdf.useMutation();

  const printBallot = useBallotPrinter({
    ...rest,

    printElement: async (element) => {
      const pdfData = await printElementToPdf(element);
      const result = await printBallotPdfMutation.mutateAsync({
        pdfData: Buffer.from(pdfData),
      });
      if (result.isErr()) {
        onPrintError(result.err());
      } else {
        onPrintCompleted();
      }
    },
  });

  useEffect(() => {
    printBallot();
  }, [printBallot]);

  return <PrintBallotScreenStatic />;
}
