import {
  enterPin,
  getCommonAccessCardId,
  logOut,
  mockCardRemoval,
  mockRaveMarkVoterCardInsertion,
} from '../support/auth';
import { closeDevDock, getVoterRegistrations } from '../support/helpers';

beforeEach(() => {
  mockCardRemoval();
  logOut();
  cy.createVoter();
  cy.visit(`/`);
  closeDevDock();
});

it('prompts voter to register', () => {
  const commonAccessCardId = getCommonAccessCardId();
  mockRaveMarkVoterCardInsertion({ commonAccessCardId });
  enterPin();
  cy.contains('Registration');
});

it('allows submitting registration', () => {
  const commonAccessCardId = getCommonAccessCardId();
  mockRaveMarkVoterCardInsertion({ commonAccessCardId });
  enterPin();

  // no registrations yet
  getVoterRegistrations().then((registrations) => {
    expect(registrations).to.have.lengthOf(0);
  });

  // register
  cy.get('[placeholder="Given Name"]').type('John');
  cy.get('[placeholder="Family Name"]').type('Doe');
  cy.contains('Submit').click();

  // // registration is created
  cy.contains('Registration is pending.');
  getVoterRegistrations().then((registrations) => {
    expect(registrations).to.have.lengthOf(1);
    expect(registrations[0]?.givenName).to.equal('John');
    expect(registrations[0]?.familyName).to.equal('Doe');
    expect(registrations[0]?.electionId).to.equal(undefined);
  });
});
