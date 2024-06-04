import { Route, Switch } from 'react-router-dom';
import { ScanScreen } from './screens/scan_screen';
import { ActionsScreen } from './screens/actions_screen';
import { SearchScreen } from './screens/search_screen';

export function AppRoot(): JSX.Element {
  return (
    <Switch>
      <Route exact path="/">
        <ActionsScreen />
      </Route>
      <Route exact path="/scan">
        <ScanScreen />
      </Route>
      <Route exact path="/search">
        <SearchScreen />
      </Route>
    </Switch>
  );
}
