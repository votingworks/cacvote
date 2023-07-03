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
import { virtualKeyboardType } from '../support/keyboard';

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
  cy.get('[data-testid="given-name"]').click();
  virtualKeyboardType('JOHN');
  cy.contains('Accept').click();

  cy.get('[data-testid="family-name"]').click();
  virtualKeyboardType('DOE');
  cy.contains('Accept').click();

  cy.get('[data-testid="address-line-1"]').click();
  virtualKeyboardType('123 MAIN ST');
  cy.contains('Accept').click();

  cy.get('[data-testid="address-line-2"]').click();
  virtualKeyboardType('APT 1');
  cy.contains('Accept').click();

  cy.get('[data-testid="city"]').click();
  virtualKeyboardType('ANYTOWN');
  cy.contains('Accept').click();

  cy.get('[data-testid="state"]').click();
  virtualKeyboardType('CA');
  cy.contains('Accept').click();

  cy.get('[data-testid="postal-code"]').click();
  virtualKeyboardType('95959');
  cy.contains('Accept').click();

  cy.get('[data-testid="state-id"]').click();
  virtualKeyboardType('B2201793');
  cy.contains('Accept').click();

  cy.contains('Submit').click();

  // registration request is created
  cy.contains('Registration is pending.');
  getVoterRegistrationRequests().then((registrations) => {
    expect(registrations).to.have.lengthOf(1);
    expect(registrations[0]?.givenName).to.equal('JOHN');
    expect(registrations[0]?.familyName).to.equal('DOE');
    expect(registrations[0]?.addressLine1).to.equal('123 MAIN ST');
    expect(registrations[0]?.addressLine2).to.equal('APT 1');
    expect(registrations[0]?.city).to.equal('ANYTOWN');
    expect(registrations[0]?.state).to.equal('CA');
    expect(registrations[0]?.postalCode).to.equal('95959');
    expect(registrations[0]?.stateId).to.equal('B2201793');
  });
});
