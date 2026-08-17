import { describe, expect, it } from 'vitest';

import {
  createBedSchema,
  createFloorSchema,
  createRoomSchema,
  setBedStatusSchema,
} from './bed-schemas';

describe('createFloorSchema', () => {
  it('accepts a valid floor', () => {
    expect(createFloorSchema.safeParse({ name: 'Ground Floor', levelOrder: 0 }).success).toBe(true);
  });

  it('rejects an empty name', () => {
    expect(createFloorSchema.safeParse({ name: '', levelOrder: 0 }).success).toBe(false);
  });

  it('rejects a negative level order', () => {
    expect(createFloorSchema.safeParse({ name: 'Ground Floor', levelOrder: -1 }).success).toBe(
      false,
    );
  });
});

describe('createRoomSchema', () => {
  function validInput(): Record<string, unknown> {
    return { floorId: 1, name: 'Ward A', roomType: 'ward', mapX: 0.5, mapY: 0.5 };
  }

  it('accepts a valid room', () => {
    expect(createRoomSchema.safeParse(validInput()).success).toBe(true);
  });

  it('rejects an invalid room type', () => {
    const result = createRoomSchema.safeParse({ ...validInput(), roomType: 'broom-closet' });
    expect(result.success).toBe(false);
  });

  it('rejects a mapX above one', () => {
    const result = createRoomSchema.safeParse({ ...validInput(), mapX: 1.5 });
    expect(result.success).toBe(false);
  });

  it('rejects a mapY below zero', () => {
    const result = createRoomSchema.safeParse({ ...validInput(), mapY: -0.1 });
    expect(result.success).toBe(false);
  });
});

describe('createBedSchema', () => {
  it('accepts a valid bed', () => {
    expect(createBedSchema.safeParse({ roomId: 1, label: 'Bed 1A' }).success).toBe(true);
  });

  it('rejects an empty label', () => {
    expect(createBedSchema.safeParse({ roomId: 1, label: '' }).success).toBe(false);
  });
});

describe('setBedStatusSchema', () => {
  it('accepts a settable status', () => {
    expect(setBedStatusSchema.safeParse({ bedId: 1, status: 'maintenance' }).success).toBe(true);
  });

  it('rejects occupied as a directly settable status', () => {
    const result = setBedStatusSchema.safeParse({ bedId: 1, status: 'occupied' });
    expect(result.success).toBe(false);
  });
});
