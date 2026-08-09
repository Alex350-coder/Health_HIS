import { Link } from '@tanstack/react-router';
import {
  BedDouble,
  Bell,
  Building2,
  LayoutDashboard,
  Map,
  Package,
  ScrollText,
  Stethoscope,
  UserCog,
  Users2,
} from 'lucide-react';

import { Button } from '@shared/ui/Button';

import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

interface AuthenticatedLayoutProps {
  userFullName: string | undefined;
  onLogout: () => void;
  logoutPending: boolean;
  children: ReactNode;
}

interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
}

// 8 top-level workflow routes (UI.md Section 2's "10 modules" minus the two nested patient tabs,
// Medical History and Billing, which are only reachable from a patient's detail page).
const PRIMARY_NAV_ITEMS: NavItem[] = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/patients', label: 'Patients', icon: Users2 },
  { to: '/hospital-map', label: 'Hospital Map', icon: Map },
  { to: '/beds', label: 'Beds', icon: BedDouble },
  { to: '/operating-rooms', label: 'Operating Rooms', icon: Stethoscope },
  { to: '/inventory', label: 'Inventory', icon: Package },
  { to: '/notifications', label: 'Notifications', icon: Bell },
  { to: '/audit', label: 'Audit', icon: ScrollText },
];

// Configuration surfaces, visually separated but not role-gated (UI.md Section 2).
const ADMIN_NAV_ITEMS: NavItem[] = [
  { to: '/facility', label: 'Facility', icon: Building2 },
  { to: '/users', label: 'Users', icon: UserCog },
];

function NavLink({ to, label, icon: Icon }: NavItem): JSX.Element {
  return (
    <Link
      to={to}
      activeOptions={{ exact: to === '/' }}
      className="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-text-secondary hover:bg-surface hover:text-text-primary"
      activeProps={{
        className: 'bg-accent/10 text-accent hover:bg-accent/10 hover:text-accent',
      }}
    >
      <Icon className="size-4" aria-hidden="true" />
      {label}
    </Link>
  );
}

function Sidebar(): JSX.Element {
  return (
    <nav
      aria-label="Main"
      className="flex w-60 shrink-0 flex-col gap-6 border-r border-border-default bg-surface-raised p-4"
    >
      <p className="px-3 text-lg font-bold text-text-primary">HIS</p>
      <div className="flex flex-col gap-1">
        {PRIMARY_NAV_ITEMS.map((item) => (
          <NavLink key={item.label} {...item} />
        ))}
      </div>
      <div className="mt-auto flex flex-col gap-1 border-t border-border-default pt-4">
        {ADMIN_NAV_ITEMS.map((item) => (
          <NavLink key={item.label} {...item} />
        ))}
      </div>
    </nav>
  );
}

interface TopBarProps {
  userFullName: string | undefined;
  onLogout: () => void;
  logoutPending: boolean;
}

function TopBar({ userFullName, onLogout, logoutPending }: TopBarProps): JSX.Element {
  return (
    <header className="flex items-center justify-end gap-4 border-b border-border-default bg-surface-raised px-6 py-3">
      <button
        type="button"
        aria-label="Notifications"
        className="relative rounded-md p-2 text-text-secondary hover:bg-surface hover:text-text-primary"
      >
        <Bell className="size-5" aria-hidden="true" />
        <span className="absolute right-1 top-1 flex size-4 items-center justify-center rounded-full bg-danger text-[10px] font-semibold text-white">
          0
        </span>
      </button>
      {userFullName !== undefined ? (
        <span className="text-sm font-medium text-text-primary">{userFullName}</span>
      ) : null}
      <Button intent="secondary" size="sm" onClick={onLogout} disabled={logoutPending}>
        Log out
      </Button>
    </header>
  );
}

/**
 * Full app shell (UI.md Section 2): persistent sidebar across all 8 top-level workflow routes plus
 * a separated admin group, and a top bar with user identity, a notification-bell stub (real unread
 * count wired in Phase 11), and logout. Presentational only — session state and the logout action
 * are owned by the caller (`shared/` never depends on a specific module, Architecture.md).
 */
export function AuthenticatedLayout({
  userFullName,
  onLogout,
  logoutPending,
  children,
}: AuthenticatedLayoutProps): JSX.Element {
  return (
    <div className="flex min-h-dvh bg-surface">
      <Sidebar />
      <div className="flex flex-1 flex-col">
        <TopBar userFullName={userFullName} onLogout={onLogout} logoutPending={logoutPending} />
        <main className="flex-1 p-6">{children}</main>
      </div>
    </div>
  );
}
