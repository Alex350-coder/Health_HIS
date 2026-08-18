import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@shared/ui/Table';

import { useReleaseBed } from '../api/bed-mutations';
import { useBedsList } from '../api/bed-queries';

import type { BedSummary } from '../types/bed-schemas';

/** The cross-patient bed board (IPC.md Section 2.1) — occupancy is read here, but a bed's
 * assignment is created from the owning patient's Beds tab, not from this page. */
export default function BedListPage(): JSX.Element {
  const query = useBedsList();

  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6 p-6">
        <h1 className="text-xl font-semibold text-text-primary">Beds</h1>
        <BedListContent query={query} />
      </main>
    </ErrorBoundary>
  );
}

function BedListContent({ query }: { query: ReturnType<typeof useBedsList> }): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (!query.isSuccess) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load the beds.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  return <BedTable beds={query.data} />;
}

function BedTable({ beds }: { beds: BedSummary[] }): JSX.Element {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Label</TableHead>
          <TableHead>Room</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Action</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {beds.map((bed) => (
          <TableRow key={bed.id}>
            <TableCell>{bed.label}</TableCell>
            <TableCell>{bed.roomId}</TableCell>
            <TableCell>{bed.status}</TableCell>
            <TableCell>
              {bed.activeAssignment ? (
                <ReleaseBedButton assignmentId={bed.activeAssignment.id} />
              ) : null}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

function ReleaseBedButton({ assignmentId }: { assignmentId: number }): JSX.Element {
  const releaseBed = useReleaseBed();

  return (
    <Button
      size="sm"
      intent="secondary"
      disabled={releaseBed.isPending}
      onClick={() => releaseBed.mutate({ bedAssignmentId: assignmentId })}
    >
      {releaseBed.isPending ? 'Releasing…' : 'Release'}
    </Button>
  );
}
