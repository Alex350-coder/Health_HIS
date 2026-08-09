import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/patient_validation.rs` exactly (Rule 17.4 — the one
 * intentionally duplicated rule set). Server-side validation is authoritative; this schema
 * exists for immediate UX feedback only.
 */
const VALID_SEXES = ['male', 'female', 'other', 'unknown'] as const;

function isWellFormedIsoDate(value: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(value);
}

function isNotInTheFuture(value: string): boolean {
  const todayIso = new Date().toISOString().slice(0, 10);
  return value <= todayIso;
}

export const dateOfBirthSchema = z
  .string()
  .refine(isWellFormedIsoDate, 'Date of birth must be a valid date (YYYY-MM-DD).')
  .refine(isNotInTheFuture, 'Date of birth cannot be in the future.');

export const sexSchema = z.enum(VALID_SEXES);

const optionalText = (maxLength: number, message: string) =>
  z.preprocess(
    (value) => (value === '' ? undefined : value),
    z.string().max(maxLength, message).optional(),
  );

export const createPatientSchema = z.object({
  medicalRecordNumber: z
    .string()
    .min(1, 'Medical record number is required.')
    .max(50, 'Medical record number is too long.'),
  fullName: z.string().min(1, 'Full name is required.').max(200, 'Full name is too long.'),
  dateOfBirth: dateOfBirthSchema,
  sex: sexSchema,
  nationalId: optionalText(50, 'National ID is too long.'),
  phone: optionalText(30, 'Phone is too long.'),
  address: optionalText(300, 'Address is too long.'),
  emergencyContactName: optionalText(200, 'Emergency contact name is too long.'),
  emergencyContactPhone: optionalText(30, 'Emergency contact phone is too long.'),
  bloodType: optionalText(10, 'Blood type is too long.'),
  allergies: optionalText(1000, 'Allergies text is too long.'),
});

export type CreatePatientInput = z.infer<typeof createPatientSchema>;

/**
 * The `optionalText` preprocess step means Zod's input type (pre-parse, what `useForm` and
 * `register` actually see) differs from its output type (post-parse, `CreatePatientInput`) for
 * every optional field. `useForm` must be typed with this input shape, not `CreatePatientInput`,
 * or `zodResolver`'s generics won't line up under `exactOptionalPropertyTypes`.
 */
export type CreatePatientFormValues = z.input<typeof createPatientSchema>;

export const updatePatientSchema = createPatientSchema.extend({
  id: z.number().int().positive(),
});

export type UpdatePatientInput = z.infer<typeof updatePatientSchema>;

export type UpdatePatientFormValues = z.input<typeof updatePatientSchema>;

export interface Patient {
  id: number;
  medicalRecordNumber: string;
  fullName: string;
  dateOfBirth: string;
  sex: string;
  nationalId: string | null;
  phone: string | null;
  address: string | null;
  emergencyContactName: string | null;
  emergencyContactPhone: string | null;
  bloodType: string | null;
  allergies: string | null;
  createdAt: string;
  updatedAt: string | null;
}
