import { DateTime } from 'luxon';
import {
  CardlessVoterUser,
  DEFAULT_OVERALL_SESSION_TIME_LIMIT_HOURS,
  ElectionManagerUser,
  PollWorkerUser,
  RaveVoterUser,
  SystemAdministratorUser,
  TEST_JURISDICTION,
} from '@votingworks/types';

export function fakeSystemAdministratorUser(
  props: Partial<SystemAdministratorUser> = {}
): SystemAdministratorUser {
  return {
    role: 'system_administrator',
    jurisdiction: TEST_JURISDICTION,
    ...props,
  };
}

export function fakeElectionManagerUser(
  props: Partial<ElectionManagerUser> = {}
): ElectionManagerUser {
  return {
    role: 'election_manager',
    jurisdiction: TEST_JURISDICTION,
    electionHash: 'election-hash',
    ...props,
  };
}

export function fakePollWorkerUser(
  props: Partial<PollWorkerUser> = {}
): PollWorkerUser {
  return {
    role: 'poll_worker',
    jurisdiction: TEST_JURISDICTION,
    electionHash: 'election-hash',
    ...props,
  };
}

export function fakeCardlessVoterUser(
  props: Partial<CardlessVoterUser> = {}
): CardlessVoterUser {
  return {
    role: 'cardless_voter',
    ballotStyleId: 'ballot-style-id',
    precinctId: 'precinct-id',
    ...props,
  };
}

export function fakeRaveVoterUser(
  props: Partial<RaveVoterUser> = {}
): RaveVoterUser {
  return {
    role: 'cacvote_voter',
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
