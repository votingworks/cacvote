import {
  enterPin,
  logOut,
  mockCardRemoval,
  mockRaveMarkVoterCardInsertion,
} from '../support/auth';
import {
  closeDevDock,
  createTestVoter,
  getVoterSelectionVotes,
} from '../support/helpers';
import { virtualKeyboardType } from '../support/keyboard';

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
  cy.window().then((win) => {
    cy.stub(win, 'print').as('print');
  });

  // get started
  mockRaveMarkVoterCardInsertion();
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
  virtualKeyboardType('MERLIN');
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

  cy.contains('Printing Your Official Ballot').should('be.visible');
  cy.get('@print').should('be.calledOnce');

  // check that we're done
  cy.contains('You’re done!').should('be.visible');

  getVoterSelectionVotes().should('deep.equal', {
    mayor: [
      {
        id: 'sherlock-holmes',
        name: 'Sherlock Holmes',
        partyIds: ['0'],
      },
    ],
    controller: [
      {
        id: 'oprah-winfrey',
        name: 'Oprah Winfrey',
        partyIds: ['1'],
      },
    ],
    attorney: [
      {
        id: 'mark-twain',
        name: 'Mark Twain',
        partyIds: ['3'],
      },
    ],
    'public-works-director': [
      {
        id: 'bill-nye',
        name: 'Bill Nye',
        partyIds: ['3'],
      },
    ],
    'chief-of-police': [
      {
        id: 'natalie-portman',
        name: 'Natalie Portman',
        partyIds: ['0'],
      },
    ],
    'parks-and-recreation-director': [
      {
        id: 'write-in-merlin',
        isWriteIn: true,
        name: 'MERLIN',
      },
    ],
    'board-of-alderman': [
      {
        id: 'steve-jobs',
        name: 'Steve Jobs',
        partyIds: ['1'],
      },
      {
        id: 'pablo-picasso',
        name: 'Pablo Picasso',
        partyIds: ['1'],
      },
      {
        id: 'helen-keller',
        name: 'Helen Keller',
        partyIds: ['0'],
      },
    ],
    'city-council': [
      {
        id: 'marie-curie',
        name: 'Marie Curie',
        partyIds: ['0'],
      },
      {
        id: 'mona-lisa',
        name: 'Mona Lisa',
        partyIds: ['3'],
      },
    ],
  });
});
