import { DateTime } from 'luxon';
import {
  DEFAULT_OVERALL_SESSION_TIME_LIMIT_HOURS,
  RaveVoterUser,
} from '@votingworks/types';

export function fakeRaveVoterUser(
  props: Partial<RaveVoterUser> = {}
): RaveVoterUser {
  return {
    role: 'rave_voter',
    commonAccessCardId: 'test-common-access-card-id',
    givenName: 'Bob',
    familyName: 'Smith',
    ...props,
  };
}

export function fakeSessionExpiresAt(): Date {
  return DateTime.now()
    .plus({ hours: DEFAULT_OVERALL_SESSION_TIME_LIMIT_HOURS })
    .toJSDate();
}
