import { H1, Main, P, Screen } from '@votingworks/ui';

export function StatusScreen(): JSX.Element {
  return (
    <Screen>
      <Main centerChild padded>
        <H1>Registration Pending</H1>
        <P>Check back here once your registration is approved.</P>
      </Main>
    </Screen>
  );
}
