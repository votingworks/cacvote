import { throwIllegalValue } from '@votingworks/basics';
import { AuthStatus } from '@votingworks/cacvote-mark-backend';
import { getAuthStatus } from './api';
import { HasCardScreen } from './screens/has_card_screen';
import { NoCardScreen } from './screens/no_card_screen';

export function AppRoot(): JSX.Element {
  const authStatusQuery = getAuthStatus.useQuery();
  const authStatus: AuthStatus = authStatusQuery.isSuccess
    ? authStatusQuery.data
    : { status: 'no_card' };

  switch (authStatus.status) {
    case 'no_card':
      return <NoCardScreen />;

    case 'has_card':
      return <HasCardScreen />;

    default:
      throwIllegalValue(authStatus);
  }
}
