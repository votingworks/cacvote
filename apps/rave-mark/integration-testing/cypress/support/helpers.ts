import { deserialize, methodUrl, serialize } from '@votingworks/grout';

// eslint-disable-next-line vx/no-import-workspace-subfolders
import {
  type VoterInfo,
  type VoterRegistrationInfo,
} from '@votingworks/rave-mark-backend/src/types/db';

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

export function getVoterRegistrations(): Cypress.Chainable<
  VoterRegistrationInfo[]
> {
  return sendGroutRequest('getVoterRegistrations', {});
}
