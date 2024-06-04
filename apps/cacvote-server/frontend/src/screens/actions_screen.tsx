import { Button, ButtonList, Main, Screen } from '@votingworks/ui';
import { useHistory } from 'react-router-dom';

export function ActionsScreen(): JSX.Element {
  const history = useHistory();

  return (
    <Screen>
      <Main>
        <ButtonList>
          <Button onPress={() => history.push('/scan')}>Scan Mail Label</Button>
        </ButtonList>
      </Main>
    </Screen>
  );
}
