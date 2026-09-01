import { describe, expect, it } from 'vitest';

import {
  createDiagnosisSchema,
  createEncounterSchema,
  createEvolutionSchema,
  createTreatmentSchema,
  dischargeEncounterSchema,
} from './medical-history-schemas';

describe('createEncounterSchema', () => {
  it('accepts a positive patient id', () => {
    expect(createEncounterSchema.safeParse({ patientId: 1 }).success).toBe(true);
  });

  it('rejects a non-positive patient id', () => {
    expect(createEncounterSchema.safeParse({ patientId: 0 }).success).toBe(false);
  });
});

describe('dischargeEncounterSchema', () => {
  it('accepts an encounter id with no discharge summary', () => {
    expect(dischargeEncounterSchema.safeParse({ encounterId: 1 }).success).toBe(true);
  });

  it('treats an empty discharge summary as absent', () => {
    const result = dischargeEncounterSchema.safeParse({ encounterId: 1, dischargeSummary: '' });
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.dischargeSummary).toBeUndefined();
    }
  });

  it('rejects a discharge summary over the max length', () => {
    const result = dischargeEncounterSchema.safeParse({
      encounterId: 1,
      dischargeSummary: 'a'.repeat(4001),
    });
    expect(result.success).toBe(false);
  });
});

describe('createDiagnosisSchema', () => {
  function validInput(): Record<string, unknown> {
    return { encounterId: 1, description: 'Type 2 diabetes' };
  }

  it('accepts a minimal valid input', () => {
    expect(createDiagnosisSchema.safeParse(validInput()).success).toBe(true);
  });

  it('rejects an empty description', () => {
    const result = createDiagnosisSchema.safeParse({ ...validInput(), description: '' });
    expect(result.success).toBe(false);
  });

  it('accepts an optional correctsDiagnosisId', () => {
    const result = createDiagnosisSchema.safeParse({ ...validInput(), correctsDiagnosisId: 5 });
    expect(result.success).toBe(true);
  });
});

describe('createTreatmentSchema', () => {
  function validInput(): Record<string, unknown> {
    return { encounterId: 1, description: 'Metformin 500mg' };
  }

  it('accepts a minimal valid input', () => {
    expect(createTreatmentSchema.safeParse(validInput()).success).toBe(true);
  });

  it('rejects an empty description', () => {
    const result = createTreatmentSchema.safeParse({ ...validInput(), description: '' });
    expect(result.success).toBe(false);
  });

  it('treats an empty dosage as absent', () => {
    const result = createTreatmentSchema.safeParse({ ...validInput(), dosage: '' });
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.dosage).toBeUndefined();
    }
  });

  it('accepts a valid inventory item and quantity pair', () => {
    const result = createTreatmentSchema.safeParse({
      ...validInput(),
      inventoryItemId: 1,
      quantity: 3,
    });
    expect(result.success).toBe(true);
  });

  it('rejects an inventory item id without a quantity', () => {
    const result = createTreatmentSchema.safeParse({ ...validInput(), inventoryItemId: 1 });
    expect(result.success).toBe(false);
  });

  it('rejects a quantity without an inventory item id', () => {
    const result = createTreatmentSchema.safeParse({ ...validInput(), quantity: 3 });
    expect(result.success).toBe(false);
  });

  it('rejects a non-positive quantity', () => {
    const result = createTreatmentSchema.safeParse({
      ...validInput(),
      inventoryItemId: 1,
      quantity: 0,
    });
    expect(result.success).toBe(false);
  });
});

describe('createEvolutionSchema', () => {
  it('accepts a valid note', () => {
    expect(
      createEvolutionSchema.safeParse({ encounterId: 1, note: 'Patient improving.' }).success,
    ).toBe(true);
  });

  it('rejects an empty note', () => {
    expect(createEvolutionSchema.safeParse({ encounterId: 1, note: '' }).success).toBe(false);
  });
});
