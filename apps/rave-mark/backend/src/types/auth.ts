import { InsertedSmartCardAuth } from '@votingworks/types';

export type AuthStatus = InsertedSmartCardAuth.AuthStatus & {
  isRaveAdmin: boolean;
};
