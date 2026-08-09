import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './Table';

describe('Table', () => {
  it('renders as a semantic table with a column header and data cells', () => {
    render(
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Room</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>12A</TableCell>
          </TableRow>
        </TableBody>
      </Table>,
    );

    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Room' })).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: '12A' })).toBeInTheDocument();
  });
});
