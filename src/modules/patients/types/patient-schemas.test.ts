import { describe, expect, it } from 'vitest';

import {
  createPatientSchema,
  dateOfBirthSchema,
  sexSchema,
  updatePatientSchema,
} from './patient-schemas';

function validInput(): Record<string, unknown> {
  return {
    medicalRecordNumber: 'MRN-0001',
    fullName: 'Ada Lovelace',
    dateOfBirth: '1990-01-01',
    sex: 'female',
  };
}

describe('dateOfBirthSchema', () => {
  it('accepts a well-formed past date', () => {
    expect(dateOfBirthSchema.safeParse('1990-01-01').success).toBe(true);
  });

  it('rejects a malformed date', () => {
    expect(dateOfBirthSchema.safeParse('01-01-1990').success).toBe(false);
  });

  it('rejects a date in the future', () => {
    expect(dateOfBirthSchema.safeParse('2999-01-01').success).toBe(false);
  });
});

describe('sexSchema', () => {
  it('accepts every valid enum value', () => {
    expect(sexSchema.safeParse('male').success).toBe(true);
    expect(sexSchema.safeParse('female').success).toBe(true);
    expect(sexSchema.safeParse('other').success).toBe(true);
    expect(sexSchema.safeParse('unknown').success).toBe(true);
  });

  it('rejects an unrecognized value', () => {
    expect(sexSchema.safeParse('unspecified').success).toBe(false);
  });
});

describe('createPatientSchema', () => {
  it('accepts a fully valid minimal input', () => {
    expect(createPatientSchema.safeParse(validInput()).success).toBe(true);
  });

  it('rejects an empty full name', () => {
    const result = createPatientSchema.safeParse({ ...validInput(), fullName: '' });
    expect(result.success).toBe(false);
  });

  it('rejects an empty medical record number', () => {
    const result = createPatientSchema.safeParse({ ...validInput(), medicalRecordNumber: '' });
    expect(result.success).toBe(false);
  });

  it('treats an empty optional field as absent', () => {
    const result = createPatientSchema.safeParse({ ...validInput(), phone: '' });
    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.phone).toBeUndefined();
    }
  });
});

describe('updatePatientSchema', () => {
  it('requires a positive integer id in addition to the create fields', () => {
    expect(updatePatientSchema.safeParse({ ...validInput(), id: 1 }).success).toBe(true);
    expect(updatePatientSchema.safeParse(validInput()).success).toBe(false);
  });
});
