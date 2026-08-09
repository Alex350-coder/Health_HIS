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

interface VirtualRowProps<T> {
  item: T;
  index: number;
  virtualRow: VirtualItem;
  getRowKey: (item: T, index: number) => string | number;
  renderRow: (item: T, index: number) => ReactNode;
}

function VirtualRow<T>({
  item,
  index,
  virtualRow,
  getRowKey,
  renderRow,
}: VirtualRowProps<T>): JSX.Element {
  return (
    <tr
      key={getRowKey(item, index)}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        width: '100%',
        height: virtualRow.size,
        transform: `translateY(${virtualRow.start}px)`,
        display: 'table',
        tableLayout: 'fixed',
      }}
      className="border-b border-border-default"
    >
      {renderRow(item, index)}
    </tr>
  );
}

interface VirtualTableBodyProps<T> {
  items: readonly T[];
  virtualRows: VirtualItem[];
  totalSize: number;
  getRowKey: (item: T, index: number) => string | number;
  renderRow: (item: T, index: number) => ReactNode;
}

function VirtualTableBody<T>({
  items,
  virtualRows,
  totalSize,
  getRowKey,
  renderRow,
}: VirtualTableBodyProps<T>): JSX.Element {
  return (
    <tbody style={{ height: totalSize, position: 'relative', display: 'block' }}>
      {virtualRows.map((virtualRow) => {
        const item = items[virtualRow.index];
        if (item === undefined) {
          return null;
        }
        return (
          <VirtualRow
            key={getRowKey(item, virtualRow.index)}
            item={item}
            index={virtualRow.index}
            virtualRow={virtualRow}
            getRowKey={getRowKey}
            renderRow={renderRow}
          />
        );
      })}
    </tbody>
  );
}

/**
 * Windowed table for datasets that can exceed 100 rows (Rule 18.1) — patients, inventory items,
 * audit log entries. Only the rows in (or near) the viewport are mounted to the DOM.
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

  return (
    <div ref={scrollRef} style={{ height }} className={cn('w-full overflow-y-auto', className)}>
      <table className="w-full border-collapse text-sm">
        <thead className="sticky top-0 z-10 border-b border-border-default bg-surface-raised">
          {renderHeader()}
        </thead>
        <VirtualTableBody
          items={items}
          virtualRows={virtualizer.getVirtualItems()}
          totalSize={virtualizer.getTotalSize()}
          getRowKey={getRowKey}
          renderRow={renderRow}
        />
      </table>
    </div>
  );
}
