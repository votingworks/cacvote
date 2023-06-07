import {
  fakeElectionManagerUser,
  fakePollWorkerUser,
  fakeRaveVoterUser,
  fakeSystemAdministratorUser,
} from '@votingworks/test-utils';

import {
  areElectionManagerCardDetails,
  arePollWorkerCardDetails,
  areRaveVoterCardDetails,
  areSystemAdministratorCardDetails,
  CardDetails,
} from './card';

const systemAdministratorUser = fakeSystemAdministratorUser();
const electionManagerUser = fakeElectionManagerUser();
const pollWorkerUser = fakePollWorkerUser();
const raveVoterUser = fakeRaveVoterUser();
const systemAdministratorCardDetails: CardDetails = {
  user: systemAdministratorUser,
};
const electionManagerCardDetails: CardDetails = {
  user: electionManagerUser,
};
const pollWorkerCardDetails: CardDetails = {
  user: pollWorkerUser,
  hasPin: false,
};
const raveVoterCardDetails: CardDetails = {
  user: raveVoterUser,
};

test.each<{ cardDetails: CardDetails; result: boolean }>([
  { cardDetails: systemAdministratorCardDetails, result: true },
  { cardDetails: electionManagerCardDetails, result: false },
  { cardDetails: pollWorkerCardDetails, result: false },
  { cardDetails: raveVoterCardDetails, result: false },
])('areSystemAdministratorCardDetails', ({ cardDetails, result }) => {
  expect(areSystemAdministratorCardDetails(cardDetails)).toEqual(result);
});

test.each<{ cardDetails: CardDetails; result: boolean }>([
  { cardDetails: systemAdministratorCardDetails, result: false },
  { cardDetails: electionManagerCardDetails, result: true },
  { cardDetails: pollWorkerCardDetails, result: false },
  { cardDetails: raveVoterCardDetails, result: false },
])('areElectionManagerCardDetails', ({ cardDetails, result }) => {
  expect(areElectionManagerCardDetails(cardDetails)).toEqual(result);
});

test.each<{ cardDetails: CardDetails; result: boolean }>([
  { cardDetails: systemAdministratorCardDetails, result: false },
  { cardDetails: electionManagerCardDetails, result: false },
  { cardDetails: pollWorkerCardDetails, result: true },
  { cardDetails: raveVoterCardDetails, result: false },
])('arePollWorkerCardDetails', ({ cardDetails, result }) => {
  expect(arePollWorkerCardDetails(cardDetails)).toEqual(result);
});

test.each<{ cardDetails: CardDetails; result: boolean }>([
  { cardDetails: systemAdministratorCardDetails, result: false },
  { cardDetails: electionManagerCardDetails, result: false },
  { cardDetails: pollWorkerCardDetails, result: false },
  { cardDetails: raveVoterCardDetails, result: true },
])('areRaveVoterCardDetails', ({ cardDetails, result }) => {
  expect(areRaveVoterCardDetails(cardDetails)).toEqual(result);
});
