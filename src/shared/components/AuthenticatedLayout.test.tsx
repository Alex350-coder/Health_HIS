import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRouter,
} from '@tanstack/react-router';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AuthenticatedLayout } from './AuthenticatedLayout';

async function renderLayout(
  props: Partial<Parameters<typeof AuthenticatedLayout>[0]> = {},
): Promise<void> {
  const rootRoute = createRootRoute({
    component: () => (
      <AuthenticatedLayout
        userFullName="Ada Lovelace"
        onLogout={() => {
          /* default no-op */
        }}
        logoutPending={false}
        {...props}
      >
        <p>page content</p>
      </AuthenticatedLayout>
    ),
  });
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  });

  render(<RouterProvider router={router} />);
  await screen.findByText('page content');
}

describe('AuthenticatedLayout', () => {
  it('renders the primary and admin nav links, user name and children', async () => {
    await renderLayout();

    expect(screen.getByRole('link', { name: 'Dashboard' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Patients' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Hospital Map' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Beds' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Operating Rooms' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Inventory' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Notifications' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Audit' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Facility' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Users' })).toBeInTheDocument();
    expect(screen.getByText('Ada Lovelace')).toBeInTheDocument();
  });

  it('renders a notification bell with an unread-count stub', async () => {
    await renderLayout();

    expect(screen.getByRole('button', { name: 'Notifications' })).toBeInTheDocument();
  });

  it('calls onLogout when the logout button is clicked', async () => {
    const user = userEvent.setup();
    const onLogout = vi.fn();
    await renderLayout({ onLogout });

    await user.click(screen.getByRole('button', { name: 'Log out' }));

    expect(onLogout).toHaveBeenCalledTimes(1);
  });

  it('disables the logout button while pending', async () => {
    await renderLayout({ logoutPending: true });

    expect(screen.getByRole('button', { name: 'Log out' })).toBeDisabled();
  });
});
