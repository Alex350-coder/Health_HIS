import { Badge } from '@shared/ui/Badge';
import { Card, CardContent, CardHeader, CardTitle } from '@shared/ui/Card';
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
    <dl className="grid grid-cols-3 gap-2 text-sm">
      <div className="flex flex-col items-center rounded-md border border-border-default p-3">
        <dt className="text-text-secondary">Available</dt>
        <dd className="text-lg font-semibold text-text-primary">{availableCount}</dd>
      </div>
      <div className="flex flex-col items-center rounded-md border border-border-default p-3">
        <dt className="text-text-secondary">Occupied</dt>
        <dd className="text-lg font-semibold text-text-primary">{occupiedCount}</dd>
      </div>
      <div className="flex flex-col items-center rounded-md border border-border-default p-3">
        <dt className="text-text-secondary">Maintenance</dt>
        <dd className="text-lg font-semibold text-text-primary">{maintenanceCount}</dd>
      </div>
    </dl>
  );
}

function RoomBedsList({ beds }: { beds: Bed[] }): JSX.Element {
  if (beds.length === 0) {
    return <EmptyState title="No beds in this room" />;
  }
  return (
    <ul className="grid grid-cols-2 gap-2">
      {beds.map((bed) => (
        <li
          key={bed.id}
          className="flex items-center justify-between gap-2 rounded-md border border-border-default p-2"
        >
          <span className="text-sm text-text-primary">{bed.label}</span>
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
    return <Skeleton className="h-40 w-full flex-1" />;
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
    <Card aria-label={`${query.data.room.name} details`} className="flex-1">
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle>{query.data.room.name}</CardTitle>
        <Badge status="neutral" className="capitalize">
          {query.data.room.roomType}
        </Badge>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <RoomOccupancySummary data={query.data} />
        <RoomBedsList beds={query.data.beds} />
      </CardContent>
    </Card>
  );
}
