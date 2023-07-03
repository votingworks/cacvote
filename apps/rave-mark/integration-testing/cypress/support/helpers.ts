import { deserialize, methodUrl, serialize } from '@votingworks/grout';
import { CreateTestVoterInput } from '@votingworks/rave-mark-backend';

// eslint-disable-next-line vx/no-import-workspace-subfolders
import {
  type VoterInfo,
  type VoterRegistrationRequest,
} from '@votingworks/rave-mark-backend/src/types/db';
import { Id, VotesDict } from '@votingworks/types';

/**
 * Closes the dev dock if it is open so it doesn't interfere with the test.
 * This isn't used in CI, but it's useful for local development.
 */
export function closeDevDock(): void {
  // close the dev dock so it doesn't interfere with the test
  cy.get('html').then((html) => {
    const handle = html.find('#handle');
    if (handle.parent('.closed').length === 0) {
      handle.trigger('click');
    }
  });
}

// eslint-disable-next-line vx/gts-no-return-type-only-generics
export function sendGroutRequest<Req, Res extends object>(
  method: string,
  params: Req
): Cypress.Chainable<Res> {
  return cy
    .request('POST', methodUrl(method, '/api'), JSON.parse(serialize(params)))
    .then((response) => deserialize(JSON.stringify(response.body)) as Res);
}

export function getVoterInfo(): Cypress.Chainable<VoterInfo> {
  return sendGroutRequest('getVoterInfo', {});
}

export function getVoterRegistrationRequests(): Cypress.Chainable<
  VoterRegistrationRequest[]
> {
  return sendGroutRequest('getVoterRegistrationRequests', {});
}

export function getVoterSelectionVotes(): Cypress.Chainable<VotesDict> {
  return sendGroutRequest('getTestVoterSelectionVotes', {});
}

export function createTestVoter(
  input: CreateTestVoterInput = {}
): Cypress.Chainable<Id> {
  return sendGroutRequest<CreateTestVoterInput, { commonAccessCardId: Id }>(
    'createTestVoter',
    input
  ).then(({ commonAccessCardId }) => {
    Cypress.env('commonAccessCardId', commonAccessCardId);
    return commonAccessCardId;
  });
}
