import { H3, Main, P, Prose, Screen } from '@votingworks/ui';
import styled, { keyframes } from 'styled-components';
import { Uuid } from '@votingworks/cacvote-mark-backend';
import { useEffect } from 'react';
import { ArrowDownIcon } from '../../components/arrow_down_icon';
import { MailLabelPreviewIcon } from '../../components/mail_label_preview_icon';
import { MailLabelPrinterIcon } from '../../components/mail_label_printer_icon';
import * as api from '../../api';

const CROP_OFFSET_Y = -165;
const INITIAL_LABEL_OFFSET_Y = -250;
const FINAL_LABEL_OFFSET_Y = -195;

const labelAnimationKeyframes = keyframes`
  0% {
    opacity: 1;
    transform: translateY(${INITIAL_LABEL_OFFSET_Y - CROP_OFFSET_Y}px);
  }
  60% {
    opacity: 1;
    transform: translateY(${FINAL_LABEL_OFFSET_Y - INITIAL_LABEL_OFFSET_Y}px);
  }
  90% {
    opacity: 0;
    transform: translateY(${FINAL_LABEL_OFFSET_Y - INITIAL_LABEL_OFFSET_Y}px);
  }
  91% {
    opacity: 0;
    transform: translateY(${INITIAL_LABEL_OFFSET_Y - CROP_OFFSET_Y}px);
  }
  100% {
    opacity: 1;
    transform: translateY(${INITIAL_LABEL_OFFSET_Y - CROP_OFFSET_Y}px);
  }
`;

const LabelPrintingAnimation = styled.div`
  animation: ${labelAnimationKeyframes} 3s ease-in-out infinite;
`;

const LabelPrintingAnimationContainer = styled.div`
  position: relative;
  top: ${CROP_OFFSET_Y}px;
  height: ${-FINAL_LABEL_OFFSET_Y - INITIAL_LABEL_OFFSET_Y}px;
  overflow: hidden;
`;

const ArrowDownIconOffset = styled.div`
  position: relative;
  top: ${FINAL_LABEL_OFFSET_Y}px;
  height: 0;
`;

const SpacedP = styled(P)`
  font-weight: 300;
  margin: 50px auto;
  max-width: 66%;
`;

export function PrintMailingLabelScreenStatic(): JSX.Element {
  return (
    <Screen>
      <Main centerChild padded>
        <Prose textCenter id="audiofocus">
          <H3>Printing Mail Label…</H3>
          <SpacedP>Tear off mail label from printer.</SpacedP>
          <SpacedP>You may put away your Common Access Card.</SpacedP>
          <MailLabelPrinterIcon />
          <LabelPrintingAnimationContainer>
            <LabelPrintingAnimation>
              <MailLabelPreviewIcon />
            </LabelPrintingAnimation>
          </LabelPrintingAnimationContainer>
          <ArrowDownIconOffset>
            <ArrowDownIcon />
          </ArrowDownIconOffset>
        </Prose>
      </Main>
    </Screen>
  );
}

export interface PrintMailingLabelScreenProps {
  printMailLabelJobId: Uuid;
  castBallotObjectId: Uuid;
  onPrintCompleted: () => void;
}

export function PrintMailingLabelScreen({
  printMailLabelJobId,
  castBallotObjectId,
  onPrintCompleted,
}: PrintMailingLabelScreenProps): JSX.Element {
  const printMailingLabelMutation = api.printMailingLabel.useMutation();
  const printMailingLabelMutationMutate = printMailingLabelMutation.mutate;

  useEffect(() => {
    const printerTimeout = setTimeout(() => {
      onPrintCompleted();
    }, 5000);

    printMailingLabelMutationMutate({
      printMailLabelJobId,
      castBallotObjectId,
    });

    return () => {
      clearTimeout(printerTimeout);
    };
  }, [
    castBallotObjectId,
    onPrintCompleted,
    printMailLabelJobId,
    printMailingLabelMutationMutate,
  ]);

  return <PrintMailingLabelScreenStatic />;
}
