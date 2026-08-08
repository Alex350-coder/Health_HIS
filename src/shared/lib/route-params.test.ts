import { describe, expect, it } from 'vitest';

import { parsePositiveIntParam } from './route-params';

describe('parsePositiveIntParam', () => {
  it('parses a numeric string into a positive integer', () => {
    expect(parsePositiveIntParam('42')).toBe(42);
  });

  it('throws for a non-numeric param', () => {
    expect(() => parsePositiveIntParam('abc')).toThrow();
  });

  it('throws for a negative or zero param', () => {
    expect(() => parsePositiveIntParam('-1')).toThrow();
    expect(() => parsePositiveIntParam('0')).toThrow();
  });
});
