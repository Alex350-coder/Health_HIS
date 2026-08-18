import { Badge } from '@shared/ui/Badge';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import type { Bed } from '@modules/beds/types/bed-schemas';

import { useRoomStatus } from '../api/hospital-map-queries';

import type { RoomStatus } from '../types/hospital-map-schemas';

const BED_STATUS_INTENT: Record<Bed['status'], 'success' | 'warning' | 'info'> = {
  available: 'success',
  occupied: 'info',
  maintenance: 'warning',
};

function RoomOccupancySummary({ data }: { data: RoomStatus }): JSX.Element {
  const { availableCount, occupiedCount, maintenanceCount } = data;
  return (
    <dl className="flex gap-4 text-sm">
      <div>
        <dt className="text-text-secondary">Available</dt>
        <dd>{availableCount}</dd>
      </div>
      <div>
        <dt className="text-text-secondary">Occupied</dt>
        <dd>{occupiedCount}</dd>
      </div>
      <div>
        <dt className="text-text-secondary">Maintenance</dt>
        <dd>{maintenanceCount}</dd>
      </div>
    </dl>
  );
}

function RoomBedsList({ beds }: { beds: Bed[] }): JSX.Element {
  if (beds.length === 0) {
    return <EmptyState title="No beds in this room" />;
  }
  return (
    <ul className="flex flex-col gap-2">
      {beds.map((bed) => (
        <li key={bed.id} className="flex items-center gap-2">
          <span>{bed.label}</span>
          <Badge status={BED_STATUS_INTENT[bed.status]}>{bed.status}</Badge>
        </li>
      ))}
    </ul>
  );
}

/** Read-only detail panel for the room selected on the map — no edit affordances (UI.md Section 7). */
export function RoomDetailPanel({ roomId }: { roomId: number }): JSX.Element {
  const query = useRoomStatus(roomId);

  if (query.isLoading) {
    return <Skeleton className="h-40 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState description="Could not load room status." onRetry={() => void query.refetch()} />
    );
  }
  if (!query.isSuccess) {
    return <EmptyState title="No data for this room" />;
  }

  return (
    <aside
      aria-label={`${query.data.room.name} details`}
      className="flex flex-col gap-3 rounded-lg border border-border-default p-4"
    >
      <h2 className="text-base font-semibold text-text-primary">{query.data.room.name}</h2>
      <p className="text-sm text-text-secondary">{query.data.room.roomType}</p>
      <RoomOccupancySummary data={query.data} />
      <RoomBedsList beds={query.data.beds} />
    </aside>
  );
}
