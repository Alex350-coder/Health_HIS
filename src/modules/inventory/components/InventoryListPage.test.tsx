import { invoke } from '@tauri-apps/api/core';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { renderWithQueryClient } from '@shared/test/render-with-query-client';

import InventoryListPage from './InventoryListPage';

import type { InventoryCategory, InventoryItem } from '../types/inventory-schemas';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigateMock,
}));

const mockedInvoke = vi.mocked(invoke);

const CATEGORY: InventoryCategory = { id: 1, name: 'Analgesics', kind: 'medicine' };

const LOW_STOCK_ITEM: InventoryItem = {
  id: 1,
  categoryId: 1,
  name: 'Ibuprofen 400mg',
  quantity: 2,
  unit: 'box',
  reorderThreshold: 10,
  expirationDate: null,
  location: null,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: null,
};

beforeAll(() => {
  for (const property of ['clientHeight', 'offsetHeight'] as const) {
    Object.defineProperty(HTMLElement.prototype, property, {
      configurable: true,
      value: 480,
    });
  }
  HTMLElement.prototype.getBoundingClientRect = (): DOMRect => ({
    x: 0,
    y: 0,
    width: 1024,
    height: 480,
    top: 0,
    left: 0,
    right: 1024,
    bottom: 480,
    toJSON(): unknown {
      return this;
    },
  });
});

function mockCommands(items: InventoryItem[], categories: InventoryCategory[]): void {
  mockedInvoke.mockImplementation((command: string) => {
    if (command === 'inventory_list_categories') return Promise.resolve(categories);
    if (command === 'inventory_list_items') return Promise.resolve(items);
    return Promise.reject(new Error(`unexpected command ${command}`));
  });
}

describe('InventoryListPage', () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    navigateMock.mockReset();
  });

  it('shows an empty state when there are no items', async () => {
    mockCommands([], []);
    renderWithQueryClient(<InventoryListPage />);

    expect(await screen.findByText('No inventory items yet')).toBeInTheDocument();
  });

  it('flags a low-stock item with a badge and explicit text', async () => {
    mockCommands([LOW_STOCK_ITEM], [CATEGORY]);
    renderWithQueryClient(<InventoryListPage />);

    expect(await screen.findByText('Ibuprofen 400mg')).toBeInTheDocument();
    expect(screen.getByText('Low stock')).toBeInTheDocument();
  });

  it('navigates to the item detail route when a row is clicked', async () => {
    mockCommands([LOW_STOCK_ITEM], [CATEGORY]);
    renderWithQueryClient(<InventoryListPage />);

    const cell = await screen.findByText('Ibuprofen 400mg');
    cell.click();

    expect(navigateMock).toHaveBeenCalledWith({ to: '/inventory/1' });
  });

  it('prompts to add a category before offering the add-item form', async () => {
    mockCommands([], []);
    renderWithQueryClient(<InventoryListPage />);

    expect(await screen.findByText('Add a category first.')).toBeInTheDocument();
  });

  it('creates a category through the inline form', async () => {
    const user = userEvent.setup();
    mockCommands([], []);
    mockedInvoke.mockImplementation((command: string) => {
      if (command === 'inventory_list_categories') return Promise.resolve([]);
      if (command === 'inventory_list_items') return Promise.resolve([]);
      if (command === 'inventory_create_category') return Promise.resolve(CATEGORY);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });
    renderWithQueryClient(<InventoryListPage />);

    await user.type(await screen.findByLabelText('Category name'), 'Analgesics');
    await user.click(screen.getByRole('button', { name: 'Add category' }));

    expect(mockedInvoke).toHaveBeenCalledWith('inventory_create_category', {
      input: { name: 'Analgesics', kind: 'medicine' },
    });
  });
});
