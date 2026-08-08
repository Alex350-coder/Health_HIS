import { render, screen } from '@testing-library/react';
import { beforeAll, describe, expect, it } from 'vitest';

import { TableCell, TableHead, TableRow } from './Table';
import { VirtualizedTable } from './VirtualizedTable';

// jsdom never performs layout, so every element reports a 0 client height. The virtualizer needs
// a non-zero viewport to compute which rows are "visible" — stub it for these tests only.
beforeAll(() => {
  for (const property of ['clientHeight', 'offsetHeight'] as const) {
    Object.defineProperty(HTMLElement.prototype, property, {
      configurable: true,
      value: 480,
    });
  }
  HTMLElement.prototype.getBoundingClientRect = (): DOMRect => ({
    x: 0,
    y: 0,
    width: 1024,
    height: 480,
    top: 0,
    left: 0,
    right: 1024,
    bottom: 480,
    toJSON(): unknown {
      return this;
    },
  });
});

interface Row {
  id: number;
  name: string;
}

function buildRows(count: number): Row[] {
  return Array.from({ length: count }, (_, index) => ({ id: index, name: `Item ${index}` }));
}

describe('VirtualizedTable', () => {
  it('renders a semantic table with the provided header', () => {
    render(
      <VirtualizedTable
        items={buildRows(5)}
        getRowKey={(row) => row.id}
        renderHeader={() => (
          <TableRow>
            <TableHead>Name</TableHead>
          </TableRow>
        )}
        renderRow={(row) => <TableCell>{row.name}</TableCell>}
      />,
    );

    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Name' })).toBeInTheDocument();
  });

  it('does not mount every row upfront for a large dataset (windowing)', () => {
    render(
      <VirtualizedTable
        items={buildRows(500)}
        getRowKey={(row) => row.id}
        renderHeader={() => (
          <TableRow>
            <TableHead>Name</TableHead>
          </TableRow>
        )}
        renderRow={(row) => <TableCell>{row.name}</TableCell>}
      />,
    );

    expect(screen.getAllByRole('cell').length).toBeLessThan(500);
  });
});
