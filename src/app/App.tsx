import { MutationCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { useState } from 'react';

import { ToastRegion } from '@shared/components/ToastRegion';
import { notifyMutationError } from '@shared/errors/notify-error';
import { useEventSubscription } from '@shared/hooks/use-event-subscription';
import { setUnauthorizedHandler } from '@shared/lib/api-client';

import { useSessionStore } from '@modules/auth/hooks/use-session-store';

import { createAppRouter } from './router';

function registerUnauthorizedHandler(router: ReturnType<typeof createAppRouter>): void {
  setUnauthorizedHandler(() => {
    useSessionStore.getState().clearSession();
    void router.navigate({ to: '/login' });
  });
}

function AppEventSubscriptions(): null {
  useEventSubscription();
  return null;
}

function createQueryClient(): QueryClient {
  return new QueryClient({
    mutationCache: new MutationCache({ onError: notifyMutationError }),
  });
}

export function App(): JSX.Element {
  const [queryClient] = useState(createQueryClient);
  const [router] = useState(() => {
    const appRouter = createAppRouter(queryClient);
    registerUnauthorizedHandler(appRouter);
    return appRouter;
  });

  return (
    <QueryClientProvider client={queryClient}>
      <AppEventSubscriptions />
      <ToastRegion />
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
