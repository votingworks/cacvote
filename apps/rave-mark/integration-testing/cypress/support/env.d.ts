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
