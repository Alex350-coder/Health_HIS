import { useDeactivateUser } from '../api/auth-mutations';

import type { User } from '../types/auth-schemas';

export function UserListRow({ user }: { user: User }): JSX.Element {
  const deactivateUser = useDeactivateUser();

  return (
    <li>
      {user.fullName} ({user.username}) — {user.role} — {user.isActive ? 'active' : 'inactive'}
      {user.isActive ? (
        <button
          type="button"
          onClick={() => deactivateUser.mutate(user.id)}
          disabled={deactivateUser.isPending}
        >
          Deactivate
        </button>
      ) : null}
    </li>
  );
}
