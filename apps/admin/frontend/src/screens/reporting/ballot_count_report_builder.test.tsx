import {
  electionFamousNames2021Fixtures,
  electionTwoPartyPrimaryDefinition,
  electionTwoPartyPrimaryFixtures,
} from '@votingworks/fixtures';
import userEvent from '@testing-library/user-event';
import { expectPrint } from '@votingworks/test-utils';
import { Tabulation } from '@votingworks/types';
import { ApiMock, createApiMock } from '../../../test/helpers/mock_api_client';
import { renderInAppContext } from '../../../test/render_in_app_context';
import { screen, within } from '../../../test/react_testing_library';
import { getMockCardCounts } from '../../../test/helpers/mock_results';
import { canonicalizeFilter, canonicalizeGroupBy } from '../../utils/reporting';
import { BallotCountReportBuilder } from './ballot_count_report_builder';

let apiMock: ApiMock;

beforeEach(() => {
  apiMock = createApiMock();
});

afterEach(() => {
  apiMock.assertComplete();
});

test('happy path', async () => {
  const electionDefinition = electionTwoPartyPrimaryDefinition;

  function getMockCardCountList(
    multiplier: number
  ): Tabulation.GroupList<Tabulation.CardCounts> {
    return [
      {
        precinctId: 'precinct-1',
        partyId: '0',
        ...getMockCardCounts(1 * multiplier),
      },
      {
        precinctId: 'precinct-1',
        partyId: '1',
        ...getMockCardCounts(2 * multiplier),
      },
      {
        precinctId: 'precinct-2',
        partyId: '0',
        ...getMockCardCounts(3 * multiplier),
      },
      {
        precinctId: 'precinct-2',
        partyId: '1',
        ...getMockCardCounts(4 * multiplier),
      },
    ];
  }

  apiMock.expectGetCastVoteRecordFileMode('test');
  apiMock.expectGetScannerBatches([]);
  apiMock.expectGetManualResultsMetadata([]);
  renderInAppContext(<BallotCountReportBuilder />, {
    electionDefinition,
    apiMock,
  });

  expect(screen.queryByText('Load Preview')).not.toBeInTheDocument();
  expect(screen.getButton('Print Report')).toBeDisabled();

  // Add Filter
  userEvent.click(screen.getByText('Add Filter'));
  userEvent.click(screen.getByLabelText('Select New Filter Type'));
  expect(
    within(screen.getByTestId('filter-editor')).queryByText('Party')
  ).toBeInTheDocument(); // party should be option for primaries, although we don't select it now
  userEvent.click(
    within(screen.getByTestId('filter-editor')).getByText('Voting Method')
  );
  screen.getByText('equals');
  userEvent.click(screen.getByLabelText('Select Filter Values'));
  userEvent.click(screen.getByText('Absentee'));

  await screen.findButton('Load Preview');
  expect(screen.getButton('Print Report')).not.toBeDisabled();

  // Add Group By
  userEvent.click(screen.getButton('Report By Precinct'));
  expect(screen.queryByLabelText('Report By Party')).toBeInTheDocument();
  userEvent.click(screen.getButton('Report By Party'));

  // Load Preview
  apiMock.expectGetCardCounts(
    {
      filter: canonicalizeFilter({
        votingMethods: ['absentee'],
      }),
      groupBy: canonicalizeGroupBy({
        groupByPrecinct: true,
        groupByParty: true,
      }),
    },
    getMockCardCountList(1)
  );
  userEvent.click(screen.getButton('Load Preview'));

  await screen.findByText('Unofficial Absentee Ballot Ballot Count Report');
  expect(screen.getByTestId('footer-total')).toHaveTextContent('10');
  expect(screen.queryByTestId('footer-bmd')).not.toBeInTheDocument();

  // Change Report Parameters
  userEvent.click(screen.getByLabelText('Remove Absentee'));
  userEvent.click(screen.getByLabelText('Select Filter Values'));
  userEvent.click(
    within(screen.getByTestId('filter-editor')).getByText('Precinct')
  );

  userEvent.click(screen.getByLabelText('Select Breakdown Type'));
  // without any manual counts, we should not see the manual option
  expect(screen.queryByText('Manual')).not.toBeInTheDocument();
  userEvent.click(screen.getByText('Full'));

  // Refresh Preview
  apiMock.expectGetCardCounts(
    {
      filter: canonicalizeFilter({
        votingMethods: ['precinct'],
      }),
      groupBy: canonicalizeGroupBy({
        groupByPrecinct: true,
        groupByParty: true,
      }),
    },
    getMockCardCountList(2)
  );
  userEvent.click(screen.getByText('Refresh Preview'));

  await screen.findByText('Unofficial Precinct Ballot Ballot Count Report');
  expect(screen.getByTestId('footer-total')).toHaveTextContent('20');
  // we added the breakdown, so we should have a BMD subtotal too
  expect(screen.getByTestId('footer-bmd')).toHaveTextContent('20');
  // we haven't added manual counts, so there should be no manual column
  expect(screen.queryByTestId('footer-manual')).not.toBeInTheDocument();

  // Print Report
  userEvent.click(screen.getButton('Print Report'));
  await expectPrint((printResult) => {
    printResult.getByText('Unofficial Precinct Ballot Ballot Count Report');
    expect(printResult.getByTestId('footer-total')).toHaveTextContent('20');
  });
});

test('does not show party options for non-primary elections', () => {
  const { electionDefinition } = electionFamousNames2021Fixtures;

  apiMock.expectGetCastVoteRecordFileMode('test');
  apiMock.expectGetScannerBatches([]);
  apiMock.expectGetManualResultsMetadata([]);
  renderInAppContext(<BallotCountReportBuilder />, {
    electionDefinition,
    apiMock,
  });

  expect(screen.queryByText('Load Preview')).not.toBeInTheDocument();

  // no group by
  expect(screen.queryByLabelText('Report By Party')).not.toBeInTheDocument();

  // no filter
  userEvent.click(screen.getByText('Add Filter'));
  userEvent.click(screen.getByLabelText('Select New Filter Type'));
  expect(
    within(screen.getByTestId('filter-editor')).queryByText('Party')
  ).not.toBeInTheDocument();
});

test('shows manual breakdown option when manual data', async () => {
  const { electionDefinition } = electionTwoPartyPrimaryFixtures;

  apiMock.expectGetCastVoteRecordFileMode('test');
  apiMock.expectGetScannerBatches([]);
  apiMock.expectGetManualResultsMetadata([
    {
      precinctId: 'precinct-1',
      ballotStyleId: '1M',
      votingMethod: 'absentee',
      ballotCount: 7,
      createdAt: 'mock',
    },
  ]);
  renderInAppContext(<BallotCountReportBuilder />, {
    electionDefinition,
    apiMock,
  });

  expect(screen.queryByText('Load Preview')).not.toBeInTheDocument();

  userEvent.click(screen.getButton('Report By Precinct'));
  userEvent.click(screen.getByLabelText('Select Breakdown Type'));
  userEvent.click(await screen.findByText('Manual'));

  apiMock.expectGetCardCounts(
    {
      filter: {},
      groupBy: canonicalizeGroupBy({
        groupByPrecinct: true,
      }),
    },
    [
      {
        precinctId: 'precinct-1',
        ...getMockCardCounts(1, 7),
      },
      {
        precinctId: 'precinct-2',
        ...getMockCardCounts(2),
      },
    ]
  );

  userEvent.click(screen.getByText('Load Preview'));

  expect(await screen.findByTestId('footer-manual')).toHaveTextContent('7');
  expect(screen.getByTestId('footer-total')).toHaveTextContent('10');
});