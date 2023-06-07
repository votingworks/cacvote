import { methodUrl } from '@votingworks/grout';

// Importing all of @votingworks/auth causes Cypress tests to fail since @votingworks/auth contains
// code that isn't browser-safe
// eslint-disable-next-line vx/no-import-workspace-subfolders
import { mockCard, MockFileContents } from '@votingworks/auth/src/cypress';

const PIN = '000000';

function mockCardCypress(mockFileContents: MockFileContents): void {
  mockCard(mockFileContents, cy.writeFile);
}

export function getCommonAccessCardId(): string {
  const commonAccessCardId = Cypress.env('commonAccessCardId');
  assert(
    typeof commonAccessCardId === 'string',
    `commonAccessCardId (${commonAccessCardId}) must be set`
  );
  return commonAccessCardId;
}

export function mockRaveMarkVoterCardInsertion({
  commonAccessCardId,
}: {
  commonAccessCardId: string;
}): void {
  mockCardCypress({
    cardStatus: {
      status: 'ready',
      cardDetails: {
        user: {
          role: 'rave_voter',
          commonAccessCardId,
        },
      },
    },
    pin: PIN,
  });
}

export function enterPin(): void {
  cy.contains('Enter the card PIN to unlock.');
  for (const digit of PIN) {
    cy.get(`button:contains(${digit})`).click();
  }
}

export function mockCardRemoval(): void {
  mockCardCypress({
    cardStatus: {
      status: 'no_card',
    },
  });
}

export function logOut(): void {
  cy.request('POST', methodUrl('logOut', '/api'), {});
}
