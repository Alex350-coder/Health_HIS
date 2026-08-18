import { useState } from 'react';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import { useHospitalMapLayout } from '../api/hospital-map-queries';

import { FloorMap } from './FloorMap';
import { RoomDetailPanel } from './RoomDetailPanel';

function HospitalMapContent(): JSX.Element {
  const query = useHospitalMapLayout();
  const [selectedRoomId, setSelectedRoomId] = useState<number | null>(null);

  if (query.isLoading) {
    return <Skeleton className="h-96 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState
        description="Could not load the hospital map."
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (!query.isSuccess || query.data.length === 0) {
    return (
      <EmptyState
        title="No facility configured yet"
        description="Configure floors, rooms, and beds from the Facility page."
      />
    );
  }

  return (
    <div className="flex flex-col gap-6">
      {query.data.map((floor) => (
        <div key={floor.id} className="flex flex-col gap-2 md:flex-row md:items-start md:gap-6">
          <FloorMap
            floor={floor}
            selectedRoomId={selectedRoomId}
            onSelectRoom={setSelectedRoomId}
          />
          {selectedRoomId !== null && floor.rooms.some((room) => room.id === selectedRoomId) ? (
            <RoomDetailPanel roomId={selectedRoomId} />
          ) : null}
        </div>
      ))}
    </div>
  );
}

/** Read-only visualization over Beds/Operating Rooms data — never writes (CLAUDE.md Section 3.3). */
export default function HospitalMapPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-4 p-6">
        <h1 className="text-xl font-semibold text-text-primary">Hospital Map</h1>
        <HospitalMapContent />
      </main>
    </ErrorBoundary>
  );
}
