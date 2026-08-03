import { Outlet } from '@tanstack/react-router';

import { AuthenticatedLayout } from '@shared/components/AuthenticatedLayout';

import { useLogout } from '@modules/auth/api/auth-mutations';
import { useSessionStore } from '@modules/auth/hooks/use-session-store';

/**
 * Composition root for the authenticated shell: only `app/` may import a specific module's
 * hooks (Architecture.md — modules never import each other directly).
 */
export function AuthenticatedLayoutRoute(): JSX.Element {
  const userFullName = useSessionStore((state) => state.user?.fullName);
  const clearSession = useSessionStore((state) => state.clearSession);
  const logout = useLogout();

  const handleLogout = (): void => {
    logout.mutate(undefined, { onSuccess: () => clearSession() });
  };

  return (
    <AuthenticatedLayout
      userFullName={userFullName}
      onLogout={handleLogout}
      logoutPending={logout.isPending}
    >
      <Outlet />
    </AuthenticatedLayout>
  );
}
