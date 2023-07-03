import {
  enterPin,
  getCommonAccessCardId,
  logOut,
  mockCardRemoval,
  mockRaveMarkVoterCardInsertion,
} from '../support/auth';
import {
  closeDevDock,
  createTestVoter,
  getVoterRegistrationRequests,
} from '../support/helpers';

beforeEach(() => {
  mockCardRemoval();
  logOut();
  createTestVoter();
  cy.visit(`/`);
  closeDevDock();
});

it('prompts voter to register', () => {
  const commonAccessCardId = getCommonAccessCardId();
  mockRaveMarkVoterCardInsertion({ commonAccessCardId });
  enterPin();
  cy.contains('Registration');
});

it('allows submitting registration requests', () => {
  const commonAccessCardId = getCommonAccessCardId();
  mockRaveMarkVoterCardInsertion({ commonAccessCardId });
  enterPin();

  // no registrations yet
  getVoterRegistrationRequests().then((registrations) => {
    expect(registrations).to.have.lengthOf(0);
  });

  // register
  cy.get('[data-testid="given-name"]').type('John');
  cy.get('[data-testid="family-name"]').type('Doe');
  cy.get('[data-testid="address-line-1"]').type('123 Main St');
  cy.get('[data-testid="address-line-2"]').type('Apt 1');
  cy.get('[data-testid="city"]').type('Anytown');
  cy.get('[data-testid="state"]').type('CA');
  cy.get('[data-testid="postal-code"]').type('95959');
  cy.get('[data-testid="state-id"]').type('B2201793');
  cy.contains('Submit').click();

  // registration request is created
  cy.contains('Registration is pending.');
  getVoterRegistrationRequests().then((registrations) => {
    expect(registrations).to.have.lengthOf(1);
    expect(registrations[0]?.givenName).to.equal('John');
    expect(registrations[0]?.familyName).to.equal('Doe');
    expect(registrations[0]?.addressLine1).to.equal('123 Main St');
    expect(registrations[0]?.addressLine2).to.equal('Apt 1');
    expect(registrations[0]?.city).to.equal('Anytown');
    expect(registrations[0]?.state).to.equal('CA');
    expect(registrations[0]?.postalCode).to.equal('95959');
    expect(registrations[0]?.stateId).to.equal('B2201793');
  });
});
