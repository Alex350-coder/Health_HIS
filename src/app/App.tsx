import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { useState } from 'react';

import { setUnauthorizedHandler } from '@shared/lib/api-client';

import { useSessionStore } from '@modules/auth/hooks/use-session-store';

import { createAppRouter } from './router';

function registerUnauthorizedHandler(router: ReturnType<typeof createAppRouter>): void {
  setUnauthorizedHandler(() => {
    useSessionStore.getState().clearSession();
    void router.navigate({ to: '/login' });
  });
}

export function App(): JSX.Element {
  const [queryClient] = useState(() => new QueryClient());
  const [router] = useState(() => {
    const appRouter = createAppRouter(queryClient);
    registerUnauthorizedHandler(appRouter);
    return appRouter;
  });

  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
