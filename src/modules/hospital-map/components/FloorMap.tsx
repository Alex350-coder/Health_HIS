import { cn } from '@shared/lib/cn';

import type { Room } from '@modules/beds/types/bed-schemas';

import type { FloorLayout } from '../types/hospital-map-schemas';

const MAP_SIZE = 480;
const ROOM_RADIUS = 18;

interface RoomMarkerProps {
  room: Room;
  isSelected: boolean;
  onSelect: (roomId: number) => void;
}

function RoomMarker({ room, isSelected, onSelect }: RoomMarkerProps): JSX.Element {
  const cx = room.mapX * MAP_SIZE;
  const cy = room.mapY * MAP_SIZE;

  return (
    <g>
      <circle
        cx={cx}
        cy={cy}
        r={ROOM_RADIUS}
        role="button"
        tabIndex={0}
        aria-label={room.name}
        aria-pressed={isSelected}
        className={cn(
          'cursor-pointer fill-info/20 stroke-info transition-colors hover:fill-info/40',
          isSelected && 'fill-info/60',
        )}
        onClick={() => onSelect(room.id)}
        onKeyDown={(event) => {
          if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            onSelect(room.id);
          }
        }}
      />
      <text
        x={cx}
        y={cy + ROOM_RADIUS + 12}
        textAnchor="middle"
        className="fill-text-secondary text-xs"
      >
        {room.name}
      </text>
    </g>
  );
}

interface FloorMapProps {
  floor: FloorLayout;
  selectedRoomId: number | null;
  onSelectRoom: (roomId: number) => void;
}

/** Positions each room on a normalized 0–1 grid — read-only, no drag/edit affordances. */
export function FloorMap({ floor, selectedRoomId, onSelectRoom }: FloorMapProps): JSX.Element {
  return (
    <svg
      role="img"
      aria-label={`${floor.name} map`}
      viewBox={`0 0 ${String(MAP_SIZE)} ${String(MAP_SIZE)}`}
      className="w-full max-w-xl rounded-lg border border-border-default bg-surface"
    >
      {floor.rooms.map((room) => (
        <RoomMarker
          key={room.id}
          room={room}
          isSelected={room.id === selectedRoomId}
          onSelect={onSelectRoom}
        />
      ))}
    </svg>
  );
}
