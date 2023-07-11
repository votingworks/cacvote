/* eslint-disable vx/gts-identifiers */
import {
  enterPin,
  logOut,
  mockCardRemoval,
  mockRaveMarkVoterCardInsertion,
} from '../support/auth';
import {
  closeDevDock,
  createTestVoter,
  getVoterCastVoteRecord,
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

  getVoterCastVoteRecord()
    .then((cvr) => cvr.CVRSnapshot[0]?.CVRContest)
    .should('deep.equal', [
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'mayor',
        Overvotes: 0,
        Undervotes: 0,
        WriteIns: 0,
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'sherlock-holmes',
            OptionPosition: 0,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
        ],
      },
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'controller',
        Overvotes: 0,
        Undervotes: 0,
        WriteIns: 0,
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'oprah-winfrey',
            OptionPosition: 1,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
        ],
      },
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'attorney',
        Overvotes: 0,
        Undervotes: 0,
        WriteIns: 0,
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'mark-twain',
            OptionPosition: 1,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
        ],
      },
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'public-works-director',
        Overvotes: 0,
        Undervotes: 0,
        WriteIns: 0,
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'bill-nye',
            OptionPosition: 2,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
        ],
      },
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'chief-of-police',
        Overvotes: 0,
        Undervotes: 0,
        WriteIns: 0,
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'natalie-portman',
            OptionPosition: 0,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
        ],
      },
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'parks-and-recreation-director',
        Overvotes: 0,
        Undervotes: 0,
        WriteIns: 1,
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'write-in-0',
            OptionPosition: 4,
            Status: ['needs-adjudication'],
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'unknown',
                CVRWriteIn: {
                  '@type': 'CVR.CVRWriteIn',
                  Text: 'MERLIN',
                },
              },
            ],
          },
        ],
      },
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'board-of-alderman',
        Overvotes: 0,
        Undervotes: 1,
        WriteIns: 0,
        Status: ['undervoted'],
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'steve-jobs',
            OptionPosition: 1,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'pablo-picasso',
            OptionPosition: 4,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'helen-keller',
            OptionPosition: 0,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
        ],
      },
      {
        '@type': 'CVR.CVRContest',
        ContestId: 'city-council',
        Overvotes: 0,
        Undervotes: 2,
        WriteIns: 0,
        Status: ['undervoted'],
        CVRContestSelection: [
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'marie-curie',
            OptionPosition: 0,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
          {
            '@type': 'CVR.CVRContestSelection',
            ContestSelectionId: 'mona-lisa',
            OptionPosition: 2,
            SelectionPosition: [
              {
                '@type': 'CVR.SelectionPosition',
                HasIndication: 'yes',
                NumberVotes: 1,
                IsAllocable: 'yes',
              },
            ],
          },
        ],
      },
    ]);
});
