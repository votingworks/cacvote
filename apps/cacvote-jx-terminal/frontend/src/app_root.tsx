import { useEffect } from 'react';
import { Route, Switch, useHistory } from 'react-router-dom';
import * as api from './api';
import { ElectionScreen } from './screens/election_screen';
import { ElectionsScreen } from './screens/elections_screen';
import { InsertCardScreen } from './screens/insert_card_screen';
import { VotersScreen } from './screens/voters_screen';
import { TallyScreen } from './screens/tally_screen';
import { UnauthenticatedSessionData } from './cacvote-server/session_data';

export function AppRoot(): JSX.Element {
  const history = useHistory();

  // this is just here for the side effects
  api.sessionData.useRootQuery();

  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const isUnauthenticated =
    sessionData && sessionData instanceof UnauthenticatedSessionData;

  useEffect(() => {
    if (isUnauthenticated) {
      history.replace('/');
    }
  }, [isUnauthenticated, history]);

  return (
    <Switch>
      <Route exact path="/">
        <InsertCardScreen />
      </Route>
      <Route exact path="/elections">
        <ElectionsScreen />
      </Route>
      <Route exact path="/elections/:electionId">
        <ElectionScreen />
      </Route>
      <Route exact path="/elections/:electionId/tally">
        <TallyScreen />
      </Route>
      <Route exact path="/voters">
        <VotersScreen />
      </Route>
    </Switch>
  );
}
