import { useEffect } from 'react';
import { Route, Switch, useHistory } from 'react-router-dom';
import * as api from './api';
import { PinPadModal } from './components/pin_pad_modal';
import { ElectionScreen } from './screens/election_screen';
import { ElectionsScreen } from './screens/elections_screen';
import { InsertCardScreen } from './screens/insert_card_screen';
import { VotersScreen } from './screens/voters_screen';
import { TallyScreen } from './screens/tally_screen';
import {
  AuthenticatingSessionData,
  UnauthenticatedSessionData,
} from './cacvote-server/session_data';

export function AppRoot(): JSX.Element {
  const history = useHistory();

  // this is just here for the side effects
  api.sessionData.useRootQuery();

  const authenticateMutation = api.authenticate.useMutation();
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const authenticationFailed = authenticateMutation.data === false;
  const isUnauthenticated =
    sessionData && sessionData instanceof UnauthenticatedSessionData;
  const isAuthenticating =
    sessionData && sessionData instanceof AuthenticatingSessionData;

  useEffect(() => {
    if (isUnauthenticated) {
      history.replace('/');
    }
  }, [isUnauthenticated, history]);

  if (isAuthenticating) {
    return (
      <PinPadModal
        isAuthenticating={authenticateMutation.isLoading}
        error={
          authenticationFailed ? 'Could not log in. Invalid PIN?' : undefined
        }
        onEnter={(pin) => {
          authenticateMutation.mutate(pin);
        }}
      />
    );
  }

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
