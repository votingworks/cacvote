import React, { useState, useContext, useCallback } from 'react';
import { Admin } from '@votingworks/api';
import {
  BallotPaperSize,
  Election,
  ElectionDefinition,
  ElementWithCallback,
  getPrecinctById,
  PrecinctId,
} from '@votingworks/types';
import { assert, sleep } from '@votingworks/basics';
import { LogEventId } from '@votingworks/logging';
import {
  BmdPaperBallot,
  Button,
  useCancelablePromise,
  Modal,
  Prose,
  HorizontalRule,
  printElement,
  printElementWhenReady,
  printElementToPdfWhenReady,
} from '@votingworks/ui';
import {
  isElectionManagerAuth,
  isSystemAdministratorAuth,
} from '@votingworks/utils';

import { AppContext } from '../contexts/app_context';
import { ButtonList } from '../components/button_list';
import { Loading } from '../components/loading';
import { NavigationScreen } from '../components/navigation_screen';
import { HandMarkedPaperBallot } from '../components/hand_marked_paper_ballot';
import { TestDeckTallyReport } from '../components/test_deck_tally_report';
import {
  generateTestDeckBallots,
  generateBlankBallots,
  generateOvervoteBallot,
} from '../utils/election';
import {
  getBallotLayoutPageSize,
  getBallotLayoutPageSizeReadableString,
} from '../utils/get_ballot_layout_page_size';
import { PrintButton } from '../components/print_button';
import { SaveFileToUsb, FileType } from '../components/save_file_to_usb';
import { generateDefaultReportFilename } from '../utils/save_as_pdf';

export const ONE_SIDED_PAGE_PRINT_TIME_MS = 3000;
export const TWO_SIDED_PAGE_PRINT_TIME_MS = 5000;
export const LAST_PRINT_JOB_SLEEP_MS = 5000;

interface PrecinctTallyReportProps {
  election: Election;
  precinctId: PrecinctId;
}

function PrecinctTallyReport({
  election,
  precinctId,
}: PrecinctTallyReportProps): JSX.Element {
  const ballots = generateTestDeckBallots({ election, precinctId });

  // Precinct test deck tallies should be twice that of a single test
  // deck because it counts scanning 2 test decks (BMD + HMPB)
  return (
    <TestDeckTallyReport
      election={election}
      testDeckBallots={[...ballots, ...ballots]}
      precinctId={precinctId}
    />
  );
}

interface BmdPaperBallotsProps {
  electionDefinition: ElectionDefinition;
  precinctId: PrecinctId;
  generateBallotId: () => string;
}

function getBmdPaperBallots({
  electionDefinition,
  precinctId,
  generateBallotId,
}: BmdPaperBallotsProps): JSX.Element[] {
  const { election } = electionDefinition;
  const ballots = generateTestDeckBallots({ election, precinctId });

  return ballots.map((ballot, i) => (
    <BmdPaperBallot
      ballotStyleId={ballot.ballotStyleId}
      electionDefinition={electionDefinition}
      generateBallotId={generateBallotId}
      isLiveMode={false}
      key={`ballot-${i}`} // eslint-disable-line react/no-array-index-key
      precinctId={ballot.precinctId}
      votes={ballot.votes}
    />
  ));
}

function generateHandMarkedPaperBallots({
  election,
  precinctId,
}: {
  election: Election;
  precinctId: PrecinctId;
}) {
  const ballots = [
    ...generateTestDeckBallots({ election, precinctId }),
    ...generateBlankBallots({ election, precinctId, numBlanks: 2 }),
  ];
  const overvoteBallot = generateOvervoteBallot({ election, precinctId });
  if (overvoteBallot) {
    ballots.push(overvoteBallot);
  }
  return ballots;
}

interface HandMarkedPaperBallotsProps {
  election: Election;
  electionHash: string;
  precinctId: PrecinctId;
}

function getHandMarkedPaperBallotsWithOnReadyCallback({
  election,
  electionHash,
  precinctId,
}: HandMarkedPaperBallotsProps): ElementWithCallback[] {
  const ballots = generateHandMarkedPaperBallots({
    election,
    precinctId,
  });

  return ballots.map((ballot, i) => {
    function HandMarkedPaperBallotWithCallback(
      onReady: VoidFunction
    ): JSX.Element {
      return (
        <HandMarkedPaperBallot
          ballotStyleId={ballot.ballotStyleId}
          election={election}
          electionHash={electionHash}
          ballotMode={Admin.BallotMode.Test}
          isAbsentee={false}
          precinctId={ballot.precinctId}
          locales={{ primary: 'en-US' }}
          votes={ballot.votes}
          onRendered={onReady}
          key={`ballot-${i}`} // eslint-disable-line react/no-array-index-key
        />
      );
    }

    return HandMarkedPaperBallotWithCallback;
  });
}

interface PrintIndex {
  precinctIds: PrecinctId[];
  precinctIndex: number;
  phase: 'PrintingLetter' | 'PaperChangeModal' | 'PrintingNonLetter';
}

interface PrintingModalProps {
  printIndex: PrintIndex;
  advancePrinting: (initialPrintIndex: PrintIndex) => void;
  election: Election;
}

function PrintingModal({
  printIndex,
  advancePrinting,
  election,
}: PrintingModalProps): JSX.Element {
  const currentPrecinct = getPrecinctById({
    election,
    precinctId: printIndex.precinctIds[printIndex.precinctIndex],
  });
  assert(currentPrecinct);

  if (printIndex.phase === 'PaperChangeModal') {
    return (
      <Modal
        centerContent
        content={
          <Prose textCenter>
            <h1>Change Paper</h1>
            <p>
              Load printer with{' '}
              <strong>
                {getBallotLayoutPageSizeReadableString(election)}-size paper
              </strong>
              .
            </p>
          </Prose>
        }
        actions={
          <Button onPress={() => advancePrinting(printIndex)} primary>
            {getBallotLayoutPageSizeReadableString(election, {
              capitalize: true,
            })}{' '}
            Paper Loaded, Continue Printing
          </Button>
        }
      />
    );
  }
  return (
    <Modal
      centerContent
      content={
        <Prose textCenter>
          <p>
            <Loading as="strong" wrapInProse={false}>
              {`Printing L&A Package for ${currentPrecinct.name}`}
            </Loading>
          </p>
          {printIndex.precinctIds.length > 1 && (
            <p>
              This is package {printIndex.precinctIndex + 1} of{' '}
              {printIndex.precinctIds.length}.
            </p>
          )}
          {getBallotLayoutPageSize(election) !== BallotPaperSize.Letter && (
            <p>
              {printIndex.phase === 'PrintingNonLetter'
                ? `Currently printing ${getBallotLayoutPageSizeReadableString(
                    election
                  )}-size pages.`
                : 'Currently printing letter-size pages.'}
            </p>
          )}
        </Prose>
      }
    />
  );
}

export function PrintTestDeckScreen(): JSX.Element {
  const makeCancelable = useCancelablePromise();
  const { electionDefinition, auth, logger, generateBallotId } =
    useContext(AppContext);
  assert(electionDefinition);
  assert(isElectionManagerAuth(auth) || isSystemAdministratorAuth(auth)); // TODO(auth) should this check for a specific user type
  const userRole = auth.user.role;
  const { election, electionHash } = electionDefinition;
  const [printIndex, setPrintIndex] = useState<PrintIndex>();
  const [isSaveModalOpen, setIsSaveModalOpen] = useState(false);
  const [precinctToSaveToPdf, setPrecinctToSaveToPdf] =
    useState<PrecinctId>('all');
  const currentPrecinct = getPrecinctById({
    election,
    precinctId: precinctToSaveToPdf,
  });

  const pageTitle = 'Precinct L&A Packages';

  const generatePrecinctIds = useCallback(
    (precinctId: PrecinctId) => {
      if (precinctId === 'all') {
        const sortedPrecincts = [...election.precincts].sort((a, b) =>
          a.name.localeCompare(b.name, undefined, {
            ignorePunctuation: true,
          })
        );
        return sortedPrecincts.map((p) => p.id);
      }

      return [precinctId];
    },
    [election.precincts]
  );

  const printPrecinctTallyReport = useCallback(
    async (precinctId: PrecinctId) => {
      const parties = new Set(election.ballotStyles.map((bs) => bs.partyId));
      const numParties = Math.max(parties.size, 1);

      await printElement(PrecinctTallyReport({ election, precinctId }), {
        sides: 'one-sided',
      });
      await logger.log(LogEventId.TestDeckTallyReportPrinted, userRole, {
        disposition: 'success',
        message: `Test deck tally report printed as part of L&A package for precinct ID: ${precinctId}`,
        precinctId,
      });
      await makeCancelable(sleep(numParties * ONE_SIDED_PAGE_PRINT_TIME_MS));
    },
    [election, logger, makeCancelable, userRole]
  );

  const printBmdPaperBallots = useCallback(
    async (precinctId: PrecinctId) => {
      const bmdPaperBallots = getBmdPaperBallots({
        electionDefinition,
        precinctId,
        generateBallotId,
      });
      await printElement(<React.Fragment>{bmdPaperBallots}</React.Fragment>, {
        sides: 'one-sided',
      });
      await logger.log(LogEventId.TestDeckPrinted, userRole, {
        disposition: 'success',
        message: `BMD paper ballot test deck printed as part of L&A package for precinct ID: ${precinctId}`,
        precinctId,
      });
      await makeCancelable(
        sleep(bmdPaperBallots.length * ONE_SIDED_PAGE_PRINT_TIME_MS)
      );
    },
    [electionDefinition, logger, makeCancelable, userRole, generateBallotId]
  );

  const printHandMarkedPaperBallots = useCallback(
    async (precinctId: PrecinctId, isLastPrecinct: boolean) => {
      const handMarkedPaperBallotsWithCallback =
        getHandMarkedPaperBallotsWithOnReadyCallback({
          election,
          electionHash,
          precinctId,
        });
      const numBallots = handMarkedPaperBallotsWithCallback.length;

      await printElementWhenReady(
        (onAllRendered) => {
          let numRendered = 0;
          function onRendered() {
            numRendered += 1;
            if (numRendered === numBallots) {
              onAllRendered();
            }
          }

          return (
            <React.Fragment>
              {handMarkedPaperBallotsWithCallback.map(
                (handMarkedPaperBallotWithCallback) =>
                  handMarkedPaperBallotWithCallback(onRendered)
              )}
            </React.Fragment>
          );
        },
        {
          sides: 'two-sided-long-edge',
        }
      );
      await logger.log(LogEventId.TestDeckPrinted, userRole, {
        disposition: 'success',
        message: `Hand-marked paper ballot test deck printed as part of L&A package for precinct ID: ${precinctId}`,
        precinctId,
      });

      if (!isLastPrecinct) {
        await makeCancelable(sleep(numBallots * TWO_SIDED_PAGE_PRINT_TIME_MS));
      } else {
        // For the last precinct, rather than waiting for all pages to finish printing, free up
        // the UI from the print modal earlier
        await makeCancelable(sleep(LAST_PRINT_JOB_SLEEP_MS));
      }
    },
    [election, electionHash, logger, makeCancelable, userRole]
  );

  const printLetterComponentsOfLogicAndAccuracyPackage = useCallback(
    async (precinctId: PrecinctId) => {
      const precinctIds = generatePrecinctIds(precinctId);
      const areHandMarkedPaperBallotsLetterSize =
        getBallotLayoutPageSize(election) === BallotPaperSize.Letter;

      for (const [
        currentPrecinctIndex,
        currentPrecinctId,
      ] of precinctIds.entries()) {
        setPrintIndex({
          precinctIds,
          precinctIndex: currentPrecinctIndex,
          phase: 'PrintingLetter',
        });
        await printPrecinctTallyReport(currentPrecinctId);
        await printBmdPaperBallots(currentPrecinctId);

        // Print HMPB ballots if they are letter-sized
        if (areHandMarkedPaperBallotsLetterSize) {
          await printHandMarkedPaperBallots(
            currentPrecinctId,
            currentPrecinctIndex === precinctIds.length - 1
          );
        }
      }

      if (areHandMarkedPaperBallotsLetterSize) {
        // We're done printing because everything was letter-sized
        setPrintIndex(undefined);
      } else {
        // We have to prompt user to replace paper and print the non-letter-sized ballots
        setPrintIndex({
          precinctIds,
          precinctIndex: 0,
          phase: 'PaperChangeModal',
        });
      }
    },
    [
      election,
      generatePrecinctIds,
      printBmdPaperBallots,
      printHandMarkedPaperBallots,
      printPrecinctTallyReport,
    ]
  );

  const onClickSaveLogicAndAccuracyPackageToPdf = useCallback(
    (precinctId: PrecinctId) => {
      setPrecinctToSaveToPdf(precinctId);
      setIsSaveModalOpen(true);
    },
    []
  );

  const renderLogicAndAccuracyPackageToPdfForSinglePrecinct = useCallback(
    (
      precinctId: PrecinctId,
      handMarkedPaperBallotCallbacks: ElementWithCallback[],
      onRendered: () => void
    ): JSX.Element => {
      return (
        <React.Fragment key={precinctId}>
          {PrecinctTallyReport({ election, precinctId })}
          {getBmdPaperBallots({
            electionDefinition,
            precinctId,
            generateBallotId,
          })}
          {handMarkedPaperBallotCallbacks.map(
            (handMarkedPaperBallotWithCallback) =>
              handMarkedPaperBallotWithCallback(onRendered)
          )}
        </React.Fragment>
      );
    },
    [election, electionDefinition, generateBallotId]
  );

  // printLogicAndAccuracyPackageToPdf prints the L&A package for all precincts to PDF format.
  // It returns a Promise<Uint8Array> to be consumed by SaveFileToUsb
  const printLogicAndAccuracyPackageToPdf =
    useCallback(async (): Promise<Uint8Array> => {
      const precinctIds = generatePrecinctIds(precinctToSaveToPdf);

      // If printing all precincts, render them all in a single call to printElementToPdfWhenReady.
      // Uint8Arrays can't easily be combined later without causing PDF rendering issues.

      // Prepare to render all hand-marked paper ballots across all precincts
      let numBallots = 0;
      const handMarkedPaperBallotsCallbacks = precinctIds.map((precinctId) => {
        const handMarkedPaperBallotsWithCallback =
          getHandMarkedPaperBallotsWithOnReadyCallback({
            election,
            electionHash,
            precinctId,
          });
        numBallots += handMarkedPaperBallotsWithCallback.length;
        return handMarkedPaperBallotsWithCallback;
      });

      return printElementToPdfWhenReady((onAllRendered) => {
        // Printing will wait until all ballots in all precincts have rendered
        let numRendered = 0;
        function onRendered() {
          numRendered += 1;
          if (numRendered === numBallots) {
            onAllRendered();
          }
        }

        return (
          <React.Fragment>
            {precinctIds.map((precinctId, i) => {
              const callbacksForPrecinct = handMarkedPaperBallotsCallbacks[i];
              return renderLogicAndAccuracyPackageToPdfForSinglePrecinct(
                precinctId,
                callbacksForPrecinct,
                onRendered
              );
            })}
          </React.Fragment>
        );
      });
    }, [
      precinctToSaveToPdf,
      election,
      electionHash,
      generatePrecinctIds,
      renderLogicAndAccuracyPackageToPdfForSinglePrecinct,
    ]);

  const printNonLetterComponentsOfLogicAndAccuracyPackage = useCallback(
    async (initialPrintIndex: PrintIndex) => {
      const { precinctIds } = initialPrintIndex;

      for (const [
        currentPrecinctIndex,
        currentPrecinctId,
      ] of precinctIds.entries()) {
        setPrintIndex({
          precinctIds,
          precinctIndex: currentPrecinctIndex,
          phase: 'PrintingNonLetter',
        });
        await printHandMarkedPaperBallots(
          currentPrecinctId,
          currentPrecinctIndex === precinctIds.length - 1
        );
      }
      setPrintIndex(undefined);
    },
    [printHandMarkedPaperBallots]
  );

  return (
    <React.Fragment>
      {printIndex && (
        <PrintingModal
          election={election}
          advancePrinting={printNonLetterComponentsOfLogicAndAccuracyPackage}
          printIndex={printIndex}
        />
      )}
      <NavigationScreen>
        <Prose maxWidth={false}>
          <h1>{pageTitle}</h1>

          <p>
            Print the L&A Packages for all precincts, or for a specific
            precinct, by selecting a button below.
          </p>
          <HorizontalRule />
          <p>Each Precinct L&A Package prints:</p>
          <ol>
            <li>
              A Precinct Tally Report — the expected results of the precinct.
            </li>
            <li>Pre-voted VxMark test ballots.</li>
            <li>Pre-voted hand-marked test ballots.</li>
            <li>
              Two blank hand-marked test ballots — one remains blank, one is
              hand-marked by an election official to replace a pre-voted
              hand-marked test ballot.
            </li>
            <li>One overvoted hand-marked test ballot.</li>
          </ol>
          <HorizontalRule />
          <ButtonList>
            <PrintButton
              print={() =>
                printLetterComponentsOfLogicAndAccuracyPackage('all')
              }
              useDefaultProgressModal={false}
              fullWidth
            >
              <strong>Print Packages for All Precincts</strong>
            </PrintButton>
            {window.kiosk && (
              <Button
                onPress={() => onClickSaveLogicAndAccuracyPackageToPdf('all')}
                fullWidth
              >
                Save Packages for All Precincts as PDF
              </Button>
            )}
          </ButtonList>
          <HorizontalRule />
          <ButtonList>
            {[...election.precincts]
              .sort((a, b) =>
                a.name.localeCompare(b.name, undefined, {
                  ignorePunctuation: true,
                })
              )
              .map((p) => (
                <React.Fragment key={p.id}>
                  <PrintButton
                    print={() =>
                      printLetterComponentsOfLogicAndAccuracyPackage(p.id)
                    }
                    useDefaultProgressModal={false}
                    fullWidth
                  >
                    Print {p.name}
                  </PrintButton>
                  {window.kiosk && (
                    <Button
                      onPress={() =>
                        onClickSaveLogicAndAccuracyPackageToPdf(p.id)
                      }
                      fullWidth
                    >
                      Save {p.name} to PDF
                    </Button>
                  )}
                  <p />
                </React.Fragment>
              ))}
          </ButtonList>
        </Prose>
      </NavigationScreen>
      {isSaveModalOpen && (
        <SaveFileToUsb
          onClose={() => setIsSaveModalOpen(false)}
          generateFileContent={() => printLogicAndAccuracyPackageToPdf()}
          defaultFilename={generateDefaultReportFilename(
            'test-deck-logic-and-accuracy-report',
            election,
            precinctToSaveToPdf === 'all'
              ? 'all-precincts'
              : currentPrecinct?.name
          )}
          fileType={FileType.LogicAndAccuracyPackage}
        />
      )}
    </React.Fragment>
  );
}