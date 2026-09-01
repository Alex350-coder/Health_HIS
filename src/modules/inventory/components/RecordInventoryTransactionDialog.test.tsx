import { invoke } from '@tauri-apps/api/core';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import { RecordInventoryTransactionDialog } from './RecordInventoryTransactionDialog';

import type { InventoryTransaction } from '../types/inventory-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const TRANSACTION: InventoryTransaction = {
  id: 1,
  itemId: 1,
  quantityDelta: 10,
  reason: 'restock',
  encounterId: null,
  treatmentId: null,
  performedByUserId: 1,
  createdAt: '2026-01-01T00:00:00Z',
};

describe('RecordInventoryTransactionDialog', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it('submits a transaction for the given item', async () => {
    const user = userEvent.setup();
    mockedInvoke.mockResolvedValueOnce(TRANSACTION);

    renderWithQueryClient(<RecordInventoryTransactionDialog itemId={1} />);

    await user.click(screen.getByRole('button', { name: 'Record transaction' }));
    await user.type(await screen.findByLabelText('Quantity change'), '10');
    await user.click(screen.getByRole('button', { name: 'Record' }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('inventory_record_transaction', {
        input: {
          itemId: 1,
          quantityDelta: 10,
          reason: 'restock',
        },
      });
    });
  });
});
