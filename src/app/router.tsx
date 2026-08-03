import {
  Outlet,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  lazyRouteComponent,
  redirect,
} from '@tanstack/react-router';

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

const notFoundRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '*',
  component: lazyRouteComponent(() => import('./NotFoundPage')),
});

const routeTree = rootRoute.addChildren([
  setupRoute,
  loginRoute,
  authenticatedLayoutRoute.addChildren([indexRoute, usersRoute, auditRoute]),
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
