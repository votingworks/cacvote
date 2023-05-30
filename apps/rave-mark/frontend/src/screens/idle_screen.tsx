import { H1, Main, P, Screen } from '@votingworks/ui';

export function IdleScreen(): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <H1>Welcome</H1>
        <P>Insert your CAC to begin.</P>
      </Main>
    </Screen>
  );
}
