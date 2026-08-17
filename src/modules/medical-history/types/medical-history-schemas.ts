import { z } from 'zod';

/**
 * Mirrors `src-tauri/src/validation/medical_history_validation.rs` exactly (Rule 17.4 — the one
 * intentionally duplicated rule set). Server-side validation is authoritative; this schema
 * exists for immediate UX feedback only.
 */

const optionalText = (maxLength: number, message: string) =>
  z.preprocess(
    (value) => (value === '' ? undefined : value),
    z.string().max(maxLength, message).optional(),
  );

export const createEncounterSchema = z.object({
  patientId: z.number().int().positive(),
});

export type CreateEncounterInput = z.infer<typeof createEncounterSchema>;

export const dischargeEncounterSchema = z.object({
  encounterId: z.number().int().positive(),
  dischargeSummary: optionalText(4000, 'Discharge summary is too long.'),
});

export type DischargeEncounterInput = z.infer<typeof dischargeEncounterSchema>;

export type DischargeEncounterFormValues = z.input<typeof dischargeEncounterSchema>;

export const createDiagnosisSchema = z.object({
  encounterId: z.number().int().positive(),
  description: z.string().min(1, 'Description is required.').max(2000, 'Description is too long.'),
  icdCode: optionalText(20, 'ICD code is too long.'),
  correctsDiagnosisId: z.number().int().positive().optional(),
});

export type CreateDiagnosisInput = z.infer<typeof createDiagnosisSchema>;

export type CreateDiagnosisFormValues = z.input<typeof createDiagnosisSchema>;

export const createTreatmentSchema = z.object({
  encounterId: z.number().int().positive(),
  diagnosisId: z.number().int().positive().optional(),
  description: z.string().min(1, 'Description is required.').max(2000, 'Description is too long.'),
  dosage: optionalText(200, 'Dosage is too long.'),
  correctsTreatmentId: z.number().int().positive().optional(),
});

export type CreateTreatmentInput = z.infer<typeof createTreatmentSchema>;

export type CreateTreatmentFormValues = z.input<typeof createTreatmentSchema>;

export const createEvolutionSchema = z.object({
  encounterId: z.number().int().positive(),
  note: z.string().min(1, 'Note is required.').max(4000, 'Note is too long.'),
});

export type CreateEvolutionInput = z.infer<typeof createEvolutionSchema>;

export type CreateEvolutionFormValues = z.input<typeof createEvolutionSchema>;

export interface Encounter {
  id: number;
  patientId: number;
  status: string;
  admittedAt: string;
  dischargedAt: string | null;
  dischargeSummary: string | null;
  createdByUserId: number;
  createdAt: string;
  updatedAt: string | null;
}

export interface Diagnosis {
  id: number;
  encounterId: number;
  description: string;
  icdCode: string | null;
  registeredByUserId: number;
  correctsDiagnosisId: number | null;
  createdAt: string;
}

export interface Treatment {
  id: number;
  encounterId: number;
  diagnosisId: number | null;
  description: string;
  dosage: string | null;
  registeredByUserId: number;
  correctsTreatmentId: number | null;
  createdAt: string;
}

export interface Evolution {
  id: number;
  encounterId: number;
  note: string;
  registeredByUserId: number;
  createdAt: string;
}

export interface MedicalHistoryBundle {
  encounters: Encounter[];
  diagnoses: Diagnosis[];
  treatments: Treatment[];
  evolutions: Evolution[];
}
