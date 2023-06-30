declare namespace Cypress {
  interface CreateVoterOptions {
    /**
     * Whether or not the voter should be an admin.
     */
    isAdmin?: boolean;

    registration?: {
      /**
       * Voter's given name, i.e. first name.
       */
      givenName?: string;

      /**
       * Voter's family name, i.e. last name.
       */
      familyName?: string;

      /**
       * Voter's address line 1.
       */
      addressLine1?: string;

      /**
       * Voter's address line 2.
       */
      addressLine2?: string;

      /**
       * Voter's city.
       */
      city?: string;

      /**
       * Voter's state.
       */
      state?: string;

      /**
       * Voter's postal code.
       */
      postalCode?: string;

      /**
       * Voter's state ID.
       */
      stateId?: string;

      /**
       * Election definition as a JSON string.
       */
      electionData?: string;
    };
  }

  interface Chainable<Subject> {
    /**
     * Creates a voter record in the database and populates the Cypress environment
     * with the voter's CAC ID under the key `commonAccessCardId`.
     */
    createVoter(options?: CreateVoterOptions): Chainable<Subject>;
  }

  interface Cypress {
    env(key: 'commonAccessCardId'): string;
  }
}
