import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/auth_validation.rs` exactly (Rule 17.4 — the one
 * intentionally duplicated rule set). Server-side validation is authoritative; this schema
 * exists for immediate UX feedback only.
 */
const VALID_ROLES = ['physician', 'nurse', 'admin', 'pharmacy', 'lab', 'receptionist'] as const;

export const usernameSchema = z
  .string()
  .min(3, 'Username must be at least 3 characters.')
  .max(50, 'Username must be at most 50 characters.')
  .regex(/^[A-Za-z0-9._-]+$/, "Username may only contain letters, numbers, '.', '_' and '-'.");

export const passwordSchema = z
  .string()
  .min(12, 'Password must be at least 12 characters.')
  .regex(/[A-Z]/, 'Password must contain an uppercase letter.')
  .regex(/[a-z]/, 'Password must contain a lowercase letter.')
  .regex(/[0-9]/, 'Password must contain a digit.')
  .regex(/[^A-Za-z0-9]/, 'Password must contain a symbol.');

export const roleSchema = z.enum(VALID_ROLES);

export const createUserSchema = z.object({
  fullName: z.string().min(1, 'Full name is required.').max(200, 'Full name is too long.'),
  username: usernameSchema,
  password: passwordSchema,
  role: roleSchema,
});

export type CreateUserInput = z.infer<typeof createUserSchema>;

export const loginSchema = z.object({
  username: z.string().min(1, 'Username is required.'),
  password: z.string().min(1, 'Password is required.'),
});

export type LoginInput = z.infer<typeof loginSchema>;

export interface User {
  id: number;
  fullName: string;
  username: string;
  role: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string | null;
}

export interface SessionResponse {
  token: string;
  user: User;
}
