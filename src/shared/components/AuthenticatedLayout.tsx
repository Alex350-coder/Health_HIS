import { Link } from '@tanstack/react-router';

import type { ReactNode } from 'react';

interface AuthenticatedLayoutProps {
  userFullName: string | undefined;
  onLogout: () => void;
  logoutPending: boolean;
  children: ReactNode;
}

/**
 * Minimal top bar + nav shared by every authenticated page (Plan.md — Phase 2 provides just
 * enough shell plumbing for Phase 3 to re-skin, not the full design system). Presentational only
 * — session state and the logout action are owned by the caller (`shared/` never depends on a
 * specific module, Architecture.md).
 */
export function AuthenticatedLayout({
  userFullName,
  onLogout,
  logoutPending,
  children,
}: AuthenticatedLayoutProps): JSX.Element {
  return (
    <div>
      <header>
        <nav aria-label="Main">
          <Link to="/">Dashboard</Link>
          <Link to="/users">Users</Link>
          <Link to="/audit">Audit</Link>
        </nav>
        {userFullName ? <span>{userFullName}</span> : null}
        <button type="button" onClick={onLogout} disabled={logoutPending}>
          Log out
        </button>
      </header>
      {children}
    </div>
  );
}
