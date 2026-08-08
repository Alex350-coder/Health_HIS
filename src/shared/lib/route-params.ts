import { z } from 'zod';

const positiveIntParamSchema = z.coerce.number().int().positive();

/**
 * Parses a route param expected to be a positive integer entity id (Routes.md Section 4) —
 * rejects malformed params before any `invoke` call fires.
 */
export function parsePositiveIntParam(rawValue: string): number {
  return positiveIntParamSchema.parse(rawValue);
}
