import { Card, CardContent, CardHeader, CardTitle } from '@shared/ui/Card';

import type { FloorLayout } from '../types/hospital-map-schemas';

function roomTypeCounts(floor: FloorLayout): [string, number][] {
  const counts = new Map<string, number>();
  for (const room of floor.rooms) {
    counts.set(room.roomType, (counts.get(room.roomType) ?? 0) + 1);
  }
  return [...counts.entries()];
}

/** Default right-hand content before a room is picked — fills the panel with floor-level
 * context instead of leaving it blank (data already present in `floor`, no extra query). */
export function FloorSummaryPanel({ floor }: { floor: FloorLayout }): JSX.Element {
  const counts = roomTypeCounts(floor);

  return (
    <Card className="flex-1">
      <CardHeader>
        <CardTitle>{floor.name}</CardTitle>
        <p className="text-sm text-text-secondary">
          {floor.rooms.length} room{floor.rooms.length === 1 ? '' : 's'} on this floor
        </p>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <div className="grid grid-cols-2 gap-2">
          {counts.map(([roomType, count]) => (
            <div
              key={roomType}
              className="flex flex-col items-center rounded-md border border-border-default p-3"
            >
              <span className="text-lg font-semibold text-text-primary">{count}</span>
              <span className="text-xs capitalize text-text-secondary">{roomType}</span>
            </div>
          ))}
        </div>
        <p className="text-sm text-text-secondary">Select a room to see its beds.</p>
      </CardContent>
    </Card>
  );
}
