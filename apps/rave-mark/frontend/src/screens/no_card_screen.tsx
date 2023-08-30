import { H1, Main, P, Screen } from '@votingworks/ui';

export function NoCardScreen(): JSX.Element {
  return (
    <Screen>
      <Main centerChild>
        <H1>Welcome</H1>
        <P>Insert your Common Access Card to begin.</P>
      </Main>
    </Screen>
  );
}
