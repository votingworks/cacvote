import { useEffect } from 'react';
import { Route, Switch, useHistory } from 'react-router-dom';
import { ElectionsScreen } from './screens/elections_screen';
import { InsertCardScreen } from './screens/insert_card_screen';
import * as api from './api';
import { VotersScreen } from './screens/voters_screen';

export function AppRoot(): JSX.Element {
  const history = useHistory();

  // this is just here for the side effects
  api.sessionData.useRootQuery();

  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;

  useEffect(() => {
    if (sessionData?.type === 'unauthenticated') {
      history.replace('/');
    }
  }, [sessionData?.type, history]);

  return (
    <Switch>
      <Route exact path="/">
        <InsertCardScreen />
      </Route>
      <Route exact path="/elections">
        <ElectionsScreen />
      </Route>
      <Route exact path="/voters">
        <VotersScreen />
      </Route>
    </Switch>
  );
}
