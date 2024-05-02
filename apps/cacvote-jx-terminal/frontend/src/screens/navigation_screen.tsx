import React from 'react';
import {
  Main,
  Screen,
  H1,
  MainHeader,
  MainContent,
  Breadcrumbs,
  Route,
} from '@votingworks/ui';
import styled from 'styled-components';
import { Sidebar } from '../components/sidebar';
import * as api from '../api';
import { UnauthenticatedSessionData } from '../cacvote-server/session_data';

export const Header = styled(MainHeader)`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-left: 0.75rem;
`;

export const HeaderActions = styled.div`
  display: flex;
  gap: 0.5rem;
  align-items: center;
`;

interface Props {
  children: React.ReactNode;
  title?: string;
  parentRoutes?: Route[];
  headerActions?: React.ReactNode;
}

export function NavigationScreen({
  children,
  title,
  parentRoutes,
  headerActions,
}: Props): JSX.Element | null {
  const sessionDataQuery = api.sessionData.useQuery();
  const sessionData = sessionDataQuery.data;
  const isUnauthenticated =
    sessionData && sessionData instanceof UnauthenticatedSessionData;

  if (isUnauthenticated) {
    return null;
  }

  return (
    <Screen flexDirection="row">
      <Sidebar
        navItems={[
          { label: 'Elections', routerPath: '/elections' },
          { label: 'Voters', routerPath: '/voters' },
        ]}
      />
      <Main flexColumn>
        <Header>
          <div>
            {title && (
              <React.Fragment>
                {parentRoutes && (
                  <Breadcrumbs
                    currentTitle={title}
                    parentRoutes={parentRoutes}
                  />
                )}
                <H1>{title}</H1>
              </React.Fragment>
            )}
          </div>
          {headerActions}
        </Header>
        <MainContent>{children}</MainContent>
      </Main>
    </Screen>
  );
}
