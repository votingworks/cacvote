import React from 'react';
import { Link, useRouteMatch } from 'react-router-dom';
import {
  AppLogo,
  LeftNav,
  NavLink,
  NavList,
  NavListItem,
} from '@votingworks/ui';

export interface SidebarProps {
  navItems: readonly NavItem[];
}

export interface NavItem {
  label: React.ReactNode;
  routerPath: string;
}

export function Sidebar({ navItems }: SidebarProps): JSX.Element {
  const currentRoute = useRouteMatch();

  function isActivePath(path: string): boolean {
    return currentRoute.path.startsWith(path);
  }

  return (
    <LeftNav>
      <Link to="/">
        <AppLogo appName="Admin" />
      </Link>
      <NavList>
        {navItems.map(({ label, routerPath }) => (
          <NavListItem key={routerPath}>
            <NavLink to={routerPath} isActive={isActivePath(routerPath)}>
              {label}
            </NavLink>
          </NavListItem>
        ))}
      </NavList>
    </LeftNav>
  );
}
