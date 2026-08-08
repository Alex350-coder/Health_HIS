import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { EVENT_QUERY_MAP } from '@shared/lib/event-query-map';

import { useEventSubscription } from './use-event-subscription';

import type { UnlistenFn } from '@tauri-apps/api/event';
import type { ReactNode } from 'react';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@shared/lib/event-query-map', () => ({ EVENT_QUERY_MAP: {} }));

const mockedListen = vi.mocked(listen);
const mockedEventQueryMap = EVENT_QUERY_MAP as Record<string, string[][]>;

function renderWithClient(queryClient: QueryClient): ReturnType<typeof renderHook> {
  function wrapper({ children }: { children: ReactNode }): JSX.Element {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  }
  return renderHook(() => useEventSubscription(), { wrapper });
}

describe('useEventSubscription', () => {
  afterEach(() => {
    vi.restoreAllMocks();
    Object.keys(mockedEventQueryMap).forEach((key) => delete mockedEventQueryMap[key]);
  });

  it('subscribes to every event name present in the map', () => {
    mockedEventQueryMap['auth:user:created'] = [['auth', 'users']];
    mockedListen.mockResolvedValue(vi.fn());
    const queryClient = new QueryClient();

    renderWithClient(queryClient);

    expect(mockedListen).toHaveBeenCalledWith('auth:user:created', expect.any(Function));
  });

  it('invalidates the mapped query keys when the event fires', async () => {
    mockedEventQueryMap['auth:user:created'] = [['auth', 'users']];
    let handler: (() => void) | undefined;
    mockedListen.mockImplementation((_eventName, callback) => {
      handler = () => callback({ event: 'auth:user:created', id: 0, payload: undefined });
      return Promise.resolve(vi.fn());
    });
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

    renderWithClient(queryClient);
    await waitFor(() => expect(handler).toBeDefined());
    handler?.();

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['auth', 'users'] });
  });

  it('unlistens on unmount', async () => {
    mockedEventQueryMap['auth:user:created'] = [['auth', 'users']];
    const unlisten = vi.fn<UnlistenFn>();
    mockedListen.mockResolvedValue(unlisten);
    const queryClient = new QueryClient();

    const { unmount } = renderWithClient(queryClient);
    await waitFor(() => expect(mockedListen).toHaveBeenCalled());
    unmount();
    await waitFor(() => expect(unlisten).toHaveBeenCalledTimes(1));
  });
});
