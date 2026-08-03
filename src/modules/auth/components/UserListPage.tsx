import { ErrorBoundary } from '@shared/components/ErrorBoundary';

import { UserList } from './UserList';

export default function UserListPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <UserList />
    </ErrorBoundary>
  );
}
