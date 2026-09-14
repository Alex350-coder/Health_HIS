import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useState } from 'react';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { Button } from '@shared/ui/Button';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';

import { useHospitalMapLayout } from '../api/hospital-map-queries';

import { FloorMap } from './FloorMap';
import { FloorSummaryPanel } from './FloorSummaryPanel';
import { RoomDetailPanel } from './RoomDetailPanel';

import type { FloorLayout } from '../types/hospital-map-schemas';

interface FloorSwitcherProps {
  floor: FloorLayout;
  floorIndex: number;
  floorCount: number;
  onPrevious: () => void;
  onNext: () => void;
}

function FloorSwitcher({
  floor,
  floorIndex,
  floorCount,
  onPrevious,
  onNext,
}: FloorSwitcherProps): JSX.Element {
  return (
    <div className="flex items-center gap-3">
      <Button
        type="button"
        intent="secondary"
        size="sm"
        aria-label="Previous floor"
        disabled={floorIndex === 0}
        onClick={onPrevious}
      >
        <ChevronLeft className="h-4 w-4" />
      </Button>
      <div className="flex flex-col items-center">
        <span className="text-base font-semibold text-text-primary">{floor.name}</span>
        <span className="text-xs text-text-secondary">
          Floor {floorIndex + 1} of {floorCount}
        </span>
      </div>
      <Button
        type="button"
        intent="secondary"
        size="sm"
        aria-label="Next floor"
        disabled={floorIndex === floorCount - 1}
        onClick={onNext}
      >
        <ChevronRight className="h-4 w-4" />
      </Button>
    </div>
  );
}

interface FloorViewProps {
  floors: FloorLayout[];
  floorIndex: number;
}

function FloorView({ floors, floorIndex }: FloorViewProps): JSX.Element {
  const [index, setIndex] = useState(floorIndex);
  const [selectedRoomId, setSelectedRoomId] = useState<number | null>(null);
  const clampedIndex = Math.min(index, floors.length - 1);
  const floor = floors[clampedIndex];

  function changeFloor(nextIndex: number): void {
    setIndex(nextIndex);
    setSelectedRoomId(null);
  }

  if (!floor) {
    return (
      <EmptyState
        title="No facility configured yet"
        description="Configure floors, rooms, and beds from the Facility page."
      />
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <FloorSwitcher
        floor={floor}
        floorIndex={clampedIndex}
        floorCount={floors.length}
        onPrevious={() => changeFloor(clampedIndex - 1)}
        onNext={() => changeFloor(clampedIndex + 1)}
      />
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:gap-6">
        <FloorMap floor={floor} selectedRoomId={selectedRoomId} onSelectRoom={setSelectedRoomId} />
        {selectedRoomId !== null && floor.rooms.some((room) => room.id === selectedRoomId) ? (
          <RoomDetailPanel roomId={selectedRoomId} />
        ) : (
          <FloorSummaryPanel floor={floor} />
        )}
      </div>
    </div>
  );
}

function HospitalMapContent(): JSX.Element {
  const query = useHospitalMapLayout();

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

  return <FloorView floors={query.data} floorIndex={0} />;
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
