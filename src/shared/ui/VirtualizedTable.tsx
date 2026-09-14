import { useVirtualizer, type VirtualItem } from '@tanstack/react-virtual';
import { useRef } from 'react';

import { cn } from '@shared/lib/cn';

import type { ReactNode } from 'react';

const ROW_HEIGHT_PX = 44;
const OVERSCAN_ROWS = 8;

interface VirtualizedTableProps<T> {
  items: readonly T[];
  getRowKey: (item: T, index: number) => string | number;
  renderHeader: () => ReactNode;
  renderRow: (item: T, index: number) => ReactNode;
  className?: string;
  /** Visible viewport height — required so the virtualizer knows how many rows to mount. */
  height?: number;
}

/** A single filler row standing in for the rows above/below the mounted window. */
function SpacerRow({ height }: { height: number }): JSX.Element | null {
  if (height <= 0) {
    return null;
  }
  return (
    <tr aria-hidden="true">
      <td style={{ height, padding: 0, border: 'none' }} />
    </tr>
  );
}

interface VirtualRowsProps<T> {
  items: readonly T[];
  virtualRows: VirtualItem[];
  getRowKey: (item: T, index: number) => string | number;
  renderRow: (item: T, index: number) => ReactNode;
}

function VirtualRows<T>({
  items,
  virtualRows,
  getRowKey,
  renderRow,
}: VirtualRowsProps<T>): JSX.Element {
  return (
    <>
      {virtualRows.map((virtualRow) => {
        const item = items[virtualRow.index];
        if (item === undefined) {
          return null;
        }
        return (
          <tr key={getRowKey(item, virtualRow.index)} className="border-b border-border-default">
            {renderRow(item, virtualRow.index)}
          </tr>
        );
      })}
    </>
  );
}

/**
 * Windowed table for datasets that can exceed 100 rows (Rule 18.1) — patients, inventory items,
 * audit log entries. Only the rows in (or near) the viewport are mounted to the DOM.
 *
 * Mounted rows are real `<tr>`s inside one shared `<tbody>`, not independently absolutely
 * positioned mini-tables — the browser's own table layout algorithm keeps every row's columns
 * aligned with the header by construction. Off-screen rows are simulated with a leading/trailing
 * spacer row (their combined height standing in for the unmounted rows) instead of absolute
 * positioning, which previously let each row compute its own, independent column widths.
 */
export function VirtualizedTable<T>({
  items,
  getRowKey,
  renderHeader,
  renderRow,
  className,
  height = 480,
}: VirtualizedTableProps<T>): JSX.Element {
  const scrollRef = useRef<HTMLDivElement>(null);

  // TanStack Virtual's returned helper functions are stable by the library's own contract; the
  // React Compiler check for this is a known false positive, not a real staleness bug.
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT_PX,
    overscan: OVERSCAN_ROWS,
  });

  const virtualRows = virtualizer.getVirtualItems();
  const lastVirtualRow = virtualRows[virtualRows.length - 1];
  const paddingTop = virtualRows[0]?.start ?? 0;
  const paddingBottom = lastVirtualRow ? virtualizer.getTotalSize() - lastVirtualRow.end : 0;

  return (
    <div ref={scrollRef} style={{ height }} className={cn('w-full overflow-y-auto', className)}>
      <table className="w-full table-fixed border-collapse text-sm">
        <thead className="sticky top-0 z-10 border-b border-border-default bg-surface-raised">
          {renderHeader()}
        </thead>
        <tbody>
          <SpacerRow height={paddingTop} />
          <VirtualRows
            items={items}
            virtualRows={virtualRows}
            getRowKey={getRowKey}
            renderRow={renderRow}
          />
          <SpacerRow height={paddingBottom} />
        </tbody>
      </table>
    </div>
  );
}
