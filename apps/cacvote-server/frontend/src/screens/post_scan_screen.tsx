import { Button, Icons, Main, P, Screen } from '@votingworks/ui';

interface Props {
  onScanAgainPressed(): void;
}

export function PostScanScreen({ onScanAgainPressed }: Props): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <P>
          <Icons.Done /> Mailing label scan success!
        </P>
        <Button onPress={onScanAgainPressed}>Scan again</Button>
      </Main>
    </Screen>
  );
}
