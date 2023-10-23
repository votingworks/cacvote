import { H1, Main, PrintingBallotImage, Prose, Screen } from '@votingworks/ui';

export function PrintMailingLabelScreen(): JSX.Element {
  return (
    <Screen white>
      <Main centerChild padded>
        <Prose textCenter id="audiofocus">
          <PrintingBallotImage />
          <H1>Printing Your Mailing Label…</H1>
        </Prose>
      </Main>
    </Screen>
  );
}
