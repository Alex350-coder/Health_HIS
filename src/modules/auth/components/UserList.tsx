import { mutationErrorMessage } from '@shared/errors/error-messages';

import { useListUsers } from '../api/auth-queries';

import { CreateUserForm } from './CreateUserForm';
import { UserListRow } from './UserListRow';

/** Loading / empty / error triad per UI.md Section 6, in plain markup — `shared/ui` does not exist yet (Phase 3). */
export function UserList(): JSX.Element {
  const usersQuery = useListUsers(true);

  return (
    <section aria-label="Users">
      <h2>Users</h2>

      <CreateUserForm />

      {usersQuery.isLoading ? <p aria-busy="true">Loading users…</p> : null}

      {usersQuery.isError ? (
        <p role="alert">
          {mutationErrorMessage(usersQuery.error) ?? 'Something went wrong loading users.'}
        </p>
      ) : null}

      {usersQuery.isSuccess && usersQuery.data.length === 0 ? <p>No users yet.</p> : null}

      {usersQuery.isSuccess && usersQuery.data.length > 0 ? (
        <ul>
          {usersQuery.data.map((user) => (
            <UserListRow key={user.id} user={user} />
          ))}
        </ul>
      ) : null}
    </section>
  );
}
