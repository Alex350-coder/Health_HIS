import { MutationCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router';
import { render, type RenderResult } from '@testing-library/react';

import { notifyMutationError } from '@shared/errors/notify-error';

import type { ReactElement } from 'react';

interface RenderWithRouterOptions {
  /** Path the memory history starts on. Defaults to the route under test, `/`. */
  initialPath?: string;
  /** Path `ui` is expected to navigate to; rendered as a plain marker route. Defaults to `/destination`. */
  destinationPath?: string;
}

/**
 * Renders `ui` as the component for `initialPath` inside a real two-route TanStack Router tree,
 * so components calling `useNavigate()` (LoginForm, BootstrapForm, AuthenticatedLayoutRoute) can
 * be exercised without a `RouterProvider` ancestor error. Pairs with `renderWithQueryClient` for
 * the query-client half — kept separate so unrelated tests using `renderWithQueryClient` are not
 * forced into a router context they don't need.
 */
export function renderWithRouterAndQueryClient(
  ui: ReactElement,
  { initialPath = '/', destinationPath = '/destination' }: RenderWithRouterOptions = {},
): RenderResult {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    mutationCache: new MutationCache({ onError: notifyMutationError }),
  });

  const rootRoute = createRootRoute();
  const sourceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: initialPath,
    component: () => ui,
  });
  const destinationRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: destinationPath,
    component: () => <p>destination reached</p>,
  });

  const router = createRouter({
    routeTree: rootRoute.addChildren([sourceRoute, destinationRoute]),
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}
