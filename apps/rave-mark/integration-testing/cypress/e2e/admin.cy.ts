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
  createTestVoter({ isAdmin: true });
  cy.visit(`/`);
  closeDevDock();
});

it('shows admin screen', () => {
  mockRaveMarkVoterCardInsertion();
  enterPin();
  cy.contains('Admin');
});

it('allows transitioning to the voter flow', () => {
  mockRaveMarkVoterCardInsertion();
  enterPin();
  cy.contains('Admin');
  cy.contains('Start voter flow').click();
  cy.contains('Registration');
});

it('allows manually starting a server sync', () => {
  mockRaveMarkVoterCardInsertion();
  enterPin();

  cy.contains('Admin');
  cy.contains('Sync with server').click();
  cy.contains('Syncing…');
  cy.contains(getCommonAccessCardId());
});
