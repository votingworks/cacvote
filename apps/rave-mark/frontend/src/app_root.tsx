import { throwIllegalValue } from '@votingworks/basics';
import { InvalidCardScreen, UnlockMachineScreen } from '@votingworks/ui';
import { InsertedSmartCardAuth } from '@votingworks/types';
import { checkPin, getAuthStatus } from './api';
import { IdleScreen } from './screens/idle_screen';
import { LoggedInScreen } from './screens/logged_in_screen';

export function AppRoot(): JSX.Element {
  const authStatusQuery = getAuthStatus.useQuery();
  const authStatus = authStatusQuery.isSuccess
    ? authStatusQuery.data
    : InsertedSmartCardAuth.DEFAULT_AUTH_STATUS;
  const checkPinMutation = checkPin.useMutation();

  switch (authStatus.status) {
    case 'logged_out':
      return <IdleScreen />;

    case 'logged_in':
      return authStatus.user.role === 'rave_voter' ? (
        <LoggedInScreen />
      ) : (
        <InvalidCardScreen reason="card_error" />
      );

    case 'checking_pin':
      return (
        <UnlockMachineScreen
          auth={authStatus}
          checkPin={async (pin) => {
            try {
              await checkPinMutation.mutateAsync({ pin });
            } catch {
              // Handled by default query client error handling
            }
          }}
        />
      );

    default:
      throwIllegalValue(authStatus);
  }
}
