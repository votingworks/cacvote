/**
 * Type text using the virtual keyboard.
 */
export function virtualKeyboardType(text: string): Cypress.Chainable {
  return cy.get('[data-testid="virtual-keyboard"]').within(() => {
    for (const char of text) {
      if (char === ' ') {
        cy.contains('space').click();
      } else {
        cy.contains(char).click();
      }
    }
  });
}
