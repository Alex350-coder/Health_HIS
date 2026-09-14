import { Link } from '@tanstack/react-router';
import { BedDouble, Bell, Map, Package, ScrollText, Stethoscope, Users2 } from 'lucide-react';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { Card, CardHeader, CardTitle } from '@shared/ui/Card';

import { useSessionStore } from '@modules/auth/hooks/use-session-store';

import type { LucideIcon } from 'lucide-react';

interface QuickLink {
  to: string;
  label: string;
  description: string;
  icon: LucideIcon;
}

const QUICK_LINKS: QuickLink[] = [
  {
    to: '/patients',
    label: 'Patients',
    description: 'Register a new patient or find an existing record.',
    icon: Users2,
  },
  {
    to: '/hospital-map',
    label: 'Hospital Map',
    description: 'View floor and room occupancy at a glance.',
    icon: Map,
  },
  {
    to: '/beds',
    label: 'Beds',
    description: 'Assign or release a bed for a patient.',
    icon: BedDouble,
  },
  {
    to: '/operating-rooms',
    label: 'Operating Rooms',
    description: 'Review or schedule an operating room reservation.',
    icon: Stethoscope,
  },
  {
    to: '/inventory',
    label: 'Inventory',
    description: 'Check stock levels and record transactions.',
    icon: Package,
  },
  {
    to: '/notifications',
    label: 'Notifications',
    description: 'Review unread alerts across the system.',
    icon: Bell,
  },
  {
    to: '/audit',
    label: 'Audit',
    description: 'Inspect the tamper-evident activity log.',
    icon: ScrollText,
  },
];

function QuickLinkCard({ to, label, description, icon: Icon }: QuickLink): JSX.Element {
  return (
    <Link
      to={to}
      className="block focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent rounded-lg"
    >
      <Card className="h-full transition-colors hover:border-accent">
        <CardHeader className="flex-row items-center gap-3 space-y-0">
          <Icon className="size-5 text-accent" aria-hidden="true" />
          <div>
            <CardTitle className="text-base">{label}</CardTitle>
            <p className="text-sm text-text-secondary">{description}</p>
          </div>
        </CardHeader>
      </Card>
    </Link>
  );
}

export default function DashboardPage(): JSX.Element {
  const userFullName = useSessionStore((state) => state.user?.fullName);

  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6">
        <div>
          <h1 className="text-xl font-semibold text-text-primary">Hospital Information System</h1>
          <p className="text-sm text-text-secondary">
            {userFullName !== undefined ? `Welcome back, ${userFullName}` : 'Welcome'} — pick a
            module below to get started.
          </p>
        </div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {QUICK_LINKS.map((link) => (
            <QuickLinkCard key={link.to} {...link} />
          ))}
        </div>
      </main>
    </ErrorBoundary>
  );
}
