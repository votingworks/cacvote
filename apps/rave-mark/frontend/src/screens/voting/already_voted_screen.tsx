import { H2, Main, Screen, Text } from '@votingworks/ui';

export function AlreadyVotedScreen(): JSX.Element {
  return (
    <Screen>
      <Main centerChild padded>
        <H2>You’ve Already Voted</H2>
        <Text center small>
          If you haven’t already done so, please mail your paper ballot using
          the provided envelope and personalized mailing label.
        </Text>
      </Main>
    </Screen>
  );
}
