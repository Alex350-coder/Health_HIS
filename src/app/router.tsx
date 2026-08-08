import {
  Outlet,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  lazyRouteComponent,
  redirect,
} from '@tanstack/react-router';

import { parsePositiveIntParam } from '@shared/lib/route-params';

import { fetchBootstrapStatus } from '@modules/auth/api/auth-queries';
import { useSessionStore } from '@modules/auth/hooks/use-session-store';

import { AuthenticatedLayoutRoute } from './AuthenticatedLayoutRoute';

import type { QueryClient } from '@tanstack/react-query';
import type { Router } from '@tanstack/react-router';

interface RouterContext {
  queryClient: QueryClient;
}

/**
 * Route guard (Routes.md Section 2): a client-side convenience only. The authoritative check is
 * always server-side — every command re-validates the session (Security.md Section 4).
 */
function requireNoSession(): void {
  if (useSessionStore.getState().token) {
    redirect({ to: '/', throw: true });
  }
}

async function requireBootstrapped(queryClient: QueryClient): Promise<void> {
  const { needsBootstrap } = await queryClient.fetchQuery({
    queryKey: ['auth', 'bootstrapStatus'],
    queryFn: fetchBootstrapStatus,
  });
  if (needsBootstrap) {
    redirect({ to: '/setup', throw: true });
  }
}

const rootRoute = createRootRouteWithContext<RouterContext>()({ component: Outlet });

const setupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/setup',
  beforeLoad: async ({ context }) => {
    const { needsBootstrap } = await context.queryClient.fetchQuery({
      queryKey: ['auth', 'bootstrapStatus'],
      queryFn: fetchBootstrapStatus,
    });
    if (!needsBootstrap) {
      redirect({ to: '/login', throw: true });
    }
  },
  component: lazyRouteComponent(() => import('@modules/auth/components/BootstrapPage')),
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  beforeLoad: async ({ context }) => {
    requireNoSession();
    await requireBootstrapped(context.queryClient);
  },
  component: lazyRouteComponent(() => import('@modules/auth/components/LoginPage')),
});

const authenticatedLayoutRoute = createRoute({
  id: 'authenticated-layout',
  getParentRoute: () => rootRoute,
  beforeLoad: () => {
    if (!useSessionStore.getState().token) {
      redirect({ to: '/login', throw: true });
    }
  },
  component: AuthenticatedLayoutRoute,
});

const indexRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/',
  component: lazyRouteComponent(() => import('./DashboardPage')),
});

const usersRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/users',
  component: lazyRouteComponent(() => import('@modules/auth/components/UserListPage')),
});

const auditRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/audit',
  component: lazyRouteComponent(() => import('@modules/audit/components/AuditLogPage')),
});

const facilityRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/facility',
  component: lazyRouteComponent(() => import('@modules/beds/components/FacilityConfigPage')),
});

const patientsRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/patients',
  component: lazyRouteComponent(() => import('@modules/patients/components/PatientListPage')),
});

const patientsNewRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/patients/new',
  component: lazyRouteComponent(() => import('@modules/patients/components/PatientCreatePage')),
});

const patientDetailRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/patients/$patientId',
  params: {
    parse: (raw) => ({ patientId: parsePositiveIntParam(raw.patientId) }),
  },
  component: lazyRouteComponent(() => import('@modules/patients/components/PatientDetailPage')),
});

const patientMedicalHistoryRoute = createRoute({
  getParentRoute: () => patientDetailRoute,
  path: '/medical-history',
  component: lazyRouteComponent(
    () => import('@modules/medical-history/components/MedicalHistoryTab'),
  ),
});

const patientBillingRoute = createRoute({
  getParentRoute: () => patientDetailRoute,
  path: '/billing',
  component: lazyRouteComponent(() => import('@modules/billing/components/BillingTab')),
});

const hospitalMapRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/hospital-map',
  component: lazyRouteComponent(() => import('@modules/hospital-map/components/HospitalMapPage')),
});

const bedsRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/beds',
  component: lazyRouteComponent(() => import('@modules/beds/components/BedListPage')),
});

const operatingRoomsRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/operating-rooms',
  component: lazyRouteComponent(() => import('@modules/operating-rooms/components/OrSchedulePage')),
});

const inventoryRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/inventory',
  component: lazyRouteComponent(() => import('@modules/inventory/components/InventoryListPage')),
});

const inventoryItemDetailRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/inventory/$itemId',
  params: {
    parse: (raw) => ({ itemId: parsePositiveIntParam(raw.itemId) }),
  },
  component: lazyRouteComponent(
    () => import('@modules/inventory/components/InventoryItemDetailPage'),
  ),
});

const notificationsRoute = createRoute({
  getParentRoute: () => authenticatedLayoutRoute,
  path: '/notifications',
  component: lazyRouteComponent(
    () => import('@modules/notifications/components/NotificationListPage'),
  ),
});

const notFoundRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '*',
  component: lazyRouteComponent(() => import('./NotFoundPage')),
});

const routeTree = rootRoute.addChildren([
  setupRoute,
  loginRoute,
  authenticatedLayoutRoute.addChildren([
    indexRoute,
    usersRoute,
    auditRoute,
    facilityRoute,
    patientsRoute,
    patientsNewRoute,
    patientDetailRoute.addChildren([patientMedicalHistoryRoute, patientBillingRoute]),
    hospitalMapRoute,
    bedsRoute,
    operatingRoomsRoute,
    inventoryRoute,
    inventoryItemDetailRoute,
    notificationsRoute,
  ]),
  notFoundRoute,
]);

export function createAppRouter(queryClient: QueryClient): Router<typeof routeTree> {
  return createRouter({ routeTree, context: { queryClient } });
}

declare module '@tanstack/react-router' {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
