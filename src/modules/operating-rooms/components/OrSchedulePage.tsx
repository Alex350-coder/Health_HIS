import { useState } from 'react';

import { ErrorBoundary } from '@shared/components/ErrorBoundary';
import { mutationErrorMessage } from '@shared/errors/error-messages';
import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@shared/ui/Table';

import { useOperatingRoomsList, useOrReservationsList } from '../api/or-queries';

import { CancelReservationButton } from './CancelReservationButton';
import { ReserveOrDialog } from './ReserveOrDialog';

import type { OperatingRoom, OrReservation } from '../types/or-schemas';

/** The OR schedule view (IPC.md Section 2.1) — reservations for one operating room at a time. */
export default function OrSchedulePage(): JSX.Element {
  const operatingRoomsQuery = useOperatingRoomsList();
  const [selectedOperatingRoomId, setSelectedOperatingRoomId] = useState<number | null>(null);

  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6 p-6">
        <h1 className="text-xl font-semibold text-text-primary">Operating Rooms</h1>
        <OperatingRoomSelector
          query={operatingRoomsQuery}
          selectedOperatingRoomId={selectedOperatingRoomId}
          onSelect={setSelectedOperatingRoomId}
        />
        {selectedOperatingRoomId !== null ? (
          <ReservationSection operatingRoomId={selectedOperatingRoomId} />
        ) : null}
      </main>
    </ErrorBoundary>
  );
}

function OperatingRoomSelector({
  query,
  selectedOperatingRoomId,
  onSelect,
}: {
  query: ReturnType<typeof useOperatingRoomsList>;
  selectedOperatingRoomId: number | null;
  onSelect: (operatingRoomId: number | null) => void;
}): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-10 w-64" />;
  }
  if (!query.isSuccess) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load the operating rooms.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (query.data.length === 0) {
    return (
      <EmptyState
        title="No operating rooms yet"
        description="Promote a room to an operating room from Facility Configuration."
      />
    );
  }
  return (
    <OperatingRoomSelect
      operatingRooms={query.data}
      selectedOperatingRoomId={selectedOperatingRoomId}
      onSelect={onSelect}
    />
  );
}

function OperatingRoomSelect({
  operatingRooms,
  selectedOperatingRoomId,
  onSelect,
}: {
  operatingRooms: OperatingRoom[];
  selectedOperatingRoomId: number | null;
  onSelect: (operatingRoomId: number | null) => void;
}): JSX.Element {
  return (
    <label className="flex flex-col gap-1.5 text-sm font-medium text-text-primary">
      Operating room
      <select
        value={selectedOperatingRoomId ?? ''}
        onChange={(event) =>
          onSelect(event.target.value === '' ? null : Number(event.target.value))
        }
      >
        <option value="">Select an operating room…</option>
        {operatingRooms.map((operatingRoom) => (
          <option key={operatingRoom.id} value={operatingRoom.id}>
            {operatingRoom.name}
          </option>
        ))}
      </select>
    </label>
  );
}

function ReservationSection({ operatingRoomId }: { operatingRoomId: number }): JSX.Element {
  const reservationsQuery = useOrReservationsList(operatingRoomId);

  return (
    <section className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-text-primary">Reservations</h2>
        <ReserveOrDialog operatingRoomId={operatingRoomId} />
      </div>
      <ReservationContent query={reservationsQuery} />
    </section>
  );
}

function ReservationContent({
  query,
}: {
  query: ReturnType<typeof useOrReservationsList>;
}): JSX.Element {
  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (!query.isSuccess) {
    return (
      <ErrorState
        description={mutationErrorMessage(query.error) ?? 'Could not load the reservations.'}
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (query.data.length === 0) {
    return (
      <EmptyState
        title="No reservations yet"
        description="Reserve a time slot for this operating room."
      />
    );
  }
  return <ReservationTable reservations={query.data} />;
}

function ReservationTable({ reservations }: { reservations: OrReservation[] }): JSX.Element {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Procedure</TableHead>
          <TableHead>Start</TableHead>
          <TableHead>End</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Action</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {reservations.map((reservation) => (
          <TableRow key={reservation.id}>
            <TableCell>{reservation.procedureDescription}</TableCell>
            <TableCell>{reservation.scheduledStart}</TableCell>
            <TableCell>{reservation.scheduledEnd}</TableCell>
            <TableCell>{reservation.status}</TableCell>
            <TableCell>
              {reservation.status === 'scheduled' || reservation.status === 'in_progress' ? (
                <CancelReservationButton reservationId={reservation.id} />
              ) : null}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
