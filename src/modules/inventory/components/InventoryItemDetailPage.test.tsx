import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import InventoryItemDetailPage from './InventoryItemDetailPage';

import type {
  InventoryItem,
  InventoryTransaction,
  MaintenanceSchedule,
} from '../types/inventory-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

vi.mock('@tanstack/react-router', () => ({
  useParams: () => ({ itemId: '1' }),
}));

const mockedInvoke = vi.mocked(invoke);

const ITEM: InventoryItem = {
  id: 1,
  categoryId: 1,
  name: 'Ibuprofen 400mg',
  quantity: 2,
  unit: 'box',
  reorderThreshold: 10,
  expirationDate: null,
  location: 'Pharmacy Shelf B2',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

const TRANSACTION: InventoryTransaction = {
  id: 1,
  itemId: 1,
  quantityDelta: 50,
  reason: 'restock',
  encounterId: null,
  treatmentId: null,
  performedByUserId: 1,
  createdAt: '2026-01-01T00:00:00Z',
};

const SCHEDULE: MaintenanceSchedule = {
  id: 1,
  inventoryItemId: 1,
  scheduledDate: '2026-03-01',
  completedDate: null,
  notes: null,
  createdAt: '2026-01-01T00:00:00Z',
};

function mockCommands(
  items: InventoryItem[],
  transactions: InventoryTransaction[],
  schedules: MaintenanceSchedule[],
): void {
  mockedInvoke.mockImplementation((command: string) => {
    if (command === 'inventory_list_items') return Promise.resolve(items);
    if (command === 'inventory_list_transactions') return Promise.resolve(transactions);
    if (command === 'inventory_list_maintenance_schedules') return Promise.resolve(schedules);
    return Promise.reject(new Error(`unexpected command ${command}`));
  });
}

describe('InventoryItemDetailPage', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('shows the item header, low-stock flag, transactions, and maintenance schedule', async () => {
    mockCommands([ITEM], [TRANSACTION], [SCHEDULE]);
    renderWithQueryClient(<InventoryItemDetailPage />);

    expect(await screen.findByText('Ibuprofen 400mg')).toBeInTheDocument();
    expect(screen.getByText('Low stock')).toBeInTheDocument();
    expect(screen.getByText('restock')).toBeInTheDocument();
    expect(screen.getByText(/2026-03-01/)).toBeInTheDocument();
  });

  it('shows an empty state for transactions when there are none', async () => {
    mockCommands([ITEM], [], []);
    renderWithQueryClient(<InventoryItemDetailPage />);

    expect(await screen.findByText('No transactions yet')).toBeInTheDocument();
    expect(screen.getByText('No maintenance scheduled')).toBeInTheDocument();
  });

  it('shows a not-found state when the item does not exist', async () => {
    mockCommands([], [], []);
    renderWithQueryClient(<InventoryItemDetailPage />);

    expect(await screen.findByText('Item not found')).toBeInTheDocument();
  });
});
