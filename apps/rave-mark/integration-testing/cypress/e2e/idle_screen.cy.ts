import { mockCardRemoval } from '../support/auth';
import { closeDevDock } from '../support/helpers';

beforeEach(() => {
  mockCardRemoval();
  cy.visit('/');
  closeDevDock();
});

specify('idle screen', () => {
  cy.contains('Welcome');
});
