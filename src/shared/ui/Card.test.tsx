import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Card, CardContent, CardFooter, CardHeader, CardTitle } from './Card';

describe('Card', () => {
  it('renders header, title, content and footer slots together', () => {
    render(
      <Card>
        <CardHeader>
          <CardTitle>Bed 12A</CardTitle>
        </CardHeader>
        <CardContent>Occupied</CardContent>
        <CardFooter>
          <span>Updated 2 minutes ago</span>
        </CardFooter>
      </Card>,
    );

    expect(screen.getByRole('heading', { name: 'Bed 12A' })).toBeInTheDocument();
    expect(screen.getByText('Occupied')).toBeInTheDocument();
    expect(screen.getByText('Updated 2 minutes ago')).toBeInTheDocument();
  });
});
