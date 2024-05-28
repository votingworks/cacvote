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
import { AuthenticatedSessionData } from '../cacvote-server/session_data';

export function InsertCardScreen(): JSX.Element {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const isAuthenticated =
    sessionData && sessionData instanceof AuthenticatedSessionData;
  const history = useHistory();

  useEffect(() => {
    if (isAuthenticated) {
      history.replace('/elections');
    }
  }, [isAuthenticated, history]);

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
