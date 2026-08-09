import { MutationCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, type RenderResult } from '@testing-library/react';

import { notifyMutationError } from '@shared/errors/notify-error';

import type { ReactElement } from 'react';

/**
 * Renders `ui` inside a fresh, retry-disabled `QueryClient` — used by every form/view test that
 * touches TanStack Query. Mirrors `App.tsx`'s `MutationCache.onError` wiring so tests exercise
 * the same Conflict/Database -> toast path as production (ErrorHandling.md Section 3).
 */
export function renderWithQueryClient(ui: ReactElement): RenderResult {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    mutationCache: new MutationCache({ onError: notifyMutationError }),
  });

  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}
