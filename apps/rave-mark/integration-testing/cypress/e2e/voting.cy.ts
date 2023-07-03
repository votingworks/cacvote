import {
  enterPin,
  getCommonAccessCardId,
  logOut,
  mockCardRemoval,
  mockRaveMarkVoterCardInsertion,
} from '../support/auth';
import { closeDevDock, createTestVoter } from '../support/helpers';

beforeEach(() => {
  mockCardRemoval();
  logOut();
  cy.readFile('cypress/fixtures/electionFamousNames2021.json', null).then(
    (electionData) => {
      createTestVoter({
        registration: { electionData: electionData.toString() },
      });
    }
  );
  cy.visit(`/`);
  closeDevDock();
});

it('records votes', () => {
  // get started
  mockRaveMarkVoterCardInsertion({
    commonAccessCardId: getCommonAccessCardId(),
  });
  enterPin();
  cy.contains('Start Voting').click();

  // vote
  cy.contains('Sherlock Holmes').click();
  cy.contains('Next').click();
  cy.contains('Oprah Winfrey').click();
  cy.contains('Next').click();
  cy.contains('Mark Twain').click();
  cy.contains('Next').click();
  cy.contains('Bill Nye').click();
  cy.contains('Next').click();
  cy.contains('Natalie Portman').click();
  cy.contains('Next').click();
  cy.contains('add write-in candidate').click();
  cy.get('[data-testid="virtual-keyboard"]').within(() => {
    cy.contains('M').click();
    cy.contains('E').click();
    cy.contains('R').click();
    cy.contains('L').click();
    cy.contains('I').click();
    cy.contains('N').click();
  });
  cy.contains('Accept').click();
  cy.contains('Next').click();
  cy.contains('Steve Jobs').click();
  cy.contains('Pablo Picasso').click();
  cy.contains('Helen Keller').click();
  cy.contains('Next').click();
  cy.contains('Marie Curie').click();
  cy.contains('Mona Lisa').click();
  cy.contains('Next').click();
  cy.contains('Print My Ballot').click();

  // check that we're done
  cy.contains('You’re done!').should('be.visible');
});
