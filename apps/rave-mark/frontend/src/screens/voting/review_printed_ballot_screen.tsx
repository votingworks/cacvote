import { Button, H1, Main, P, Screen } from '@votingworks/ui';

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
        <H1>Review Your Ballot</H1>
        <P align="center">
          Please check your selections on the printed ballot paper, then select
          one of the choices below.
        </P>
        <P>
          <Button variant="done" onPress={onConfirmPrintedBallotSelections}>
            My Printed Ballot is Correct
          </Button>
        </P>
        <P>- or -</P>
        <P>
          <Button variant="warning" onPress={onRejectPrintedBallotSelections}>
            I Need to Make Changes
          </Button>
        </P>
      </Main>
    </Screen>
  );
}
