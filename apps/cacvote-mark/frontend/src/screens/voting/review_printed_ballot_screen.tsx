import { Button, H1, Main, P, Screen } from '@votingworks/ui';
import styled from 'styled-components';
import { ReviewIcon } from './review_icon';

export const IconAndBody = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: center;
  width: 74%;

  > * {
    padding: 20px;
    max-width: 65%;
  }
`;

export const ButtonsRow = styled.div`
  display: flex;
  flex-direction: row;
  margin-top: 20px;

  > * + * {
    margin-left: 20px;
  }
`;

export interface ReviewPrintedBallotScreenProps {
  onConfirm: () => void;
  onReject: () => void;
}

export function ReviewPrintedBallotScreen({
  onConfirm: onConfirmPrintedBallotSelections,
  onReject: onRejectPrintedBallotSelections,
}: ReviewPrintedBallotScreenProps): JSX.Element | null {
  return (
    <Screen>
      <Main centerChild>
        <IconAndBody>
          <ReviewIcon />
          <div>
            <H1>Review Your Ballot</H1>
            <P>
              Please check your selections on the printed ballot paper, then
              select one of the choices below.
            </P>
            <ButtonsRow>
              <Button icon="Done" onPress={onConfirmPrintedBallotSelections}>
                My Printed Ballot is Correct
              </Button>
              <Button icon="Warning" onPress={onRejectPrintedBallotSelections}>
                I Need to Make Changes
              </Button>
            </ButtonsRow>
          </div>
        </IconAndBody>
      </Main>
    </Screen>
  );
}
