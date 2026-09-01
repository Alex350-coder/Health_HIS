import { Badge } from '@shared/ui/Badge';

import type { InventoryItem } from '../types/inventory-schemas';
import type { KeyboardEvent } from 'react';

interface InventoryItemRowProps {
  item: InventoryItem;
  onSelect: () => void;
}

/**
 * One row's cells for `VirtualizedTable`'s `renderRow` contract — the `<tr>` wrapper lives there.
 * Low stock is flagged with both a badge and explicit text (Rule 19.2 — no color-only signaling).
 */
export function InventoryItemRow({ item, onSelect }: InventoryItemRowProps): JSX.Element {
  const isLowStock = item.quantity <= item.reorderThreshold;

  const handleKeyDown = (event: KeyboardEvent): void => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onSelect();
    }
  };

  return (
    <>
      <td
        className="cursor-pointer p-3 align-middle"
        role="button"
        tabIndex={0}
        aria-label={`View inventory item ${item.name}`}
        onClick={onSelect}
        onKeyDown={handleKeyDown}
      >
        {item.name}
      </td>
      <td className="cursor-pointer p-3 align-middle" onClick={onSelect}>
        {item.quantity} {item.unit}
      </td>
      <td className="cursor-pointer p-3 align-middle" onClick={onSelect}>
        {item.reorderThreshold}
      </td>
      <td className="cursor-pointer p-3 align-middle" onClick={onSelect}>
        {item.expirationDate ?? '—'}
      </td>
      <td className="cursor-pointer p-3 align-middle" onClick={onSelect}>
        {isLowStock ? (
          <Badge status="warning">Low stock</Badge>
        ) : (
          <Badge status="success">OK</Badge>
        )}
      </td>
    </>
  );
}
