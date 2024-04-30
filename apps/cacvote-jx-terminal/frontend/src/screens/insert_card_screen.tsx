import { useEffect } from 'react';
import { useHistory } from 'react-router-dom';
import {
  FullScreenIconWrapper,
  H1,
  InsertCardImage,
  Main,
  Screen,
} from '@votingworks/ui';
import * as api from '../api';

export function InsertCardScreen(): JSX.Element {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const history = useHistory();

  useEffect(() => {
    if (sessionData?.type === 'authenticated') {
      history.replace('/elections');
    }
  }, [sessionData?.type, history]);

  return (
    <Screen>
      <Main centerChild>
        <FullScreenIconWrapper>
          <InsertCardImage />
          <H1 align="center">Insert Your Card</H1>
        </FullScreenIconWrapper>
      </Main>
    </Screen>
  );
}
