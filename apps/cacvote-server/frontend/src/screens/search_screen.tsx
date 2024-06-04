import { throwIllegalValue } from '@votingworks/basics';
import {
  Button,
  H1,
  InputGroup,
  LoadingButton,
  Main,
  Screen,
  TD,
  Table,
  Text,
} from '@votingworks/ui';
import { DateTime } from 'luxon';
import { useState } from 'react';
import styled from 'styled-components';
import * as api from '../api';

const Results = styled.div`
  margin-top: 1rem;
`;

export function SearchScreen(): JSX.Element {
  const [commonAccessCardId, setCommonAccessCardId] = useState<string>();
  const searchByCommonAccessCardIdMutation =
    api.searchByCommonAccessCardId.useMutation();

  function onSearch(): void {
    if (commonAccessCardId) {
      searchByCommonAccessCardIdMutation.mutate(commonAccessCardId);
    }
  }

  const searchResult = searchByCommonAccessCardIdMutation.data;

  return (
    <Screen>
      <Main padded>
        <H1>Search by CAC ID</H1>
        <InputGroup>
          <input
            type="text"
            placeholder="CAC ID"
            disabled={searchByCommonAccessCardIdMutation.isLoading}
            value={commonAccessCardId ?? ''}
            onChange={(event) => {
              setCommonAccessCardId((event.target as HTMLInputElement).value);
            }}
            onKeyUp={(event) => {
              if (event.key === 'Enter') {
                onSearch();
              }
            }}
          />
          {searchByCommonAccessCardIdMutation.isLoading ? (
            <LoadingButton>Searching</LoadingButton>
          ) : (
            <Button onPress={onSearch} disabled={!commonAccessCardId}>
              Search
            </Button>
          )}
        </InputGroup>
        <Results>
          {searchResult?.isErr() ? (
            <Text error>{searchResult.err()}</Text>
          ) : searchResult?.isOk() ? (
            <Table>
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Election</th>
                  <th>Date</th>
                </tr>
              </thead>
              <tbody>
                {searchResult.ok().map((result) => (
                  <tr
                    key={`${result.type}${result.electionObjectId}|${result.commonAccessCardId}`}
                  >
                    <TD>
                      {result.type === 'castBallot'
                        ? '🗳️ Cast Ballot'
                        : result.type === 'scannedMailingLabelCode'
                        ? '📬 Captured Mail Label'
                        : throwIllegalValue(result)}
                    </TD>
                    <TD>{result.election ?? 'Unknown'}</TD>
                    <TD>
                      {DateTime.fromISO(result.createdAt).toFormat(
                        'LLL dd yyyy'
                      )}
                    </TD>
                  </tr>
                ))}
              </tbody>
            </Table>
          ) : null}
        </Results>
      </Main>
    </Screen>
  );
}
