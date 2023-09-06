import { H2, Main, Screen, Text } from '@votingworks/ui';

export function PostVoteScreen(): JSX.Element {
  return (
    <Screen>
      <Main centerChild padded>
        <H2>Mail Your Paper Ballot</H2>
        <Text center small>
          Please mail your paper ballot to:
        </Text>
        <Text center>
          123 Main St.
          <br />
          Anytown, USA 12345
        </Text>
        <Text center small>
          Remove your card when you’re ready to end your voting session.
        </Text>
      </Main>
    </Screen>
  );
}
