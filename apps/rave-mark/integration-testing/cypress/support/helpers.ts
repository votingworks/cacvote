import { deserialize, methodUrl, serialize } from '@votingworks/grout';
import { CreateTestVoterInput } from '@votingworks/rave-mark-backend';

// eslint-disable-next-line vx/no-import-workspace-subfolders
import { type RegistrationRequest } from '@votingworks/rave-mark-backend/src/types/db';
import { CVR, Id } from '@votingworks/types';

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

export function getRegistrationRequests(): Cypress.Chainable<
  RegistrationRequest[]
> {
  return sendGroutRequest('getRegistrationRequests', {});
}

export function getVoterCastVoteRecord(): Cypress.Chainable<CVR.CVR> {
  return sendGroutRequest('getTestVoterCastVoteRecord', {});
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
