import { assertDefined } from '@votingworks/basics';
import { H2, P } from '@votingworks/ui';
import { useParams } from 'react-router-dom';
import * as api from '../api';
import { NavigationScreen } from './navigation_screen';

export interface TallyScreenParams {
  electionId: string;
}

export function TallyScreen(): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const { electionId } = useParams<TallyScreenParams>();

  if (sessionData?.type !== 'authenticated' || !electionId) {
    return null;
  }

  const electionPresenter = assertDefined(
    sessionData.elections.find((e) => e.id === electionId)
  );

  return (
    <NavigationScreen
      title="Tally Election"
      parentRoutes={[
        { title: 'Elections', path: '/elections' },
        {
          title: electionPresenter.election.electionDefinition.election.title,
          path: `/elections/${electionId}`,
        },
      ]}
    >
      <H2>Encrypted Election Tally</H2>
      <P>
        TODO: Once all the ballots have been received, this section should allow
        a user to generate an encrypted tally of the election. This will
        disallow syncing any further ballots from the server. The encrypted
        tally should be downloadable from this section.
      </P>
      <P>
        TODO: If the encrypted tally has already been generated, just show it
        here (i.e. for download).
      </P>

      <H2>Decrypted Election Tally</H2>
      <P>
        TODO: Once the encrypted election tally exists, this section should
        allow a user to decrypt the tally and to either show it or allow
        downloading it here.
      </P>
      <P>TODO: If the tally has already been decrypted, just show it here.</P>
    </NavigationScreen>
  );
}
