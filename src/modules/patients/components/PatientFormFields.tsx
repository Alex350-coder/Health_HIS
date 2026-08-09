import { FormField } from '@shared/ui/FormField';

import type { CreatePatientFormValues, UpdatePatientFormValues } from '../types/patient-schemas';
import type { FieldErrors, UseFormRegister } from 'react-hook-form';

const SEX_OPTIONS = ['female', 'male', 'other', 'unknown'] as const;

type PatientFieldValues = CreatePatientFormValues | UpdatePatientFormValues;

/**
 * `FieldErrors<T>` for a generic `T extends A | B` resolves each accessor to a union of
 * conditional types the compiler can't collapse back to `string | undefined`. Every field below
 * is shared verbatim between `CreatePatientFormValues` and `UpdatePatientFormValues`, so casting
 * to a concrete `FieldErrors<CreatePatientFormValues>` is safe and keeps `.message` typed as a
 * plain string.
 */
function asPatientFieldErrors<T extends PatientFieldValues>(
  errors: FieldErrors<T>,
): FieldErrors<CreatePatientFormValues> {
  return errors as unknown as FieldErrors<CreatePatientFormValues>;
}

interface PatientFormFieldsProps<T extends PatientFieldValues> {
  idPrefix: string;
  register: UseFormRegister<T>;
  errors: FieldErrors<T>;
}

/** The full patient field set, shared by `PatientCreateForm` and `PatientEditForm`. */
export function PatientFormFields<T extends PatientFieldValues>({
  idPrefix,
  register,
  errors,
}: PatientFormFieldsProps<T>): JSX.Element {
  const fieldErrors = asPatientFieldErrors(errors);
  return (
    <>
      <FormField
        id={`${idPrefix}-medical-record-number`}
        label="Medical record number"
        error={fieldErrors.medicalRecordNumber?.message}
      >
        <input
          id={`${idPrefix}-medical-record-number`}
          type="text"
          {...register('medicalRecordNumber' as never)}
        />
      </FormField>
      <FormField
        id={`${idPrefix}-full-name`}
        label="Full name"
        error={fieldErrors.fullName?.message}
      >
        <input id={`${idPrefix}-full-name`} type="text" {...register('fullName' as never)} />
      </FormField>
      <FormField
        id={`${idPrefix}-date-of-birth`}
        label="Date of birth"
        error={fieldErrors.dateOfBirth?.message}
      >
        <input id={`${idPrefix}-date-of-birth`} type="date" {...register('dateOfBirth' as never)} />
      </FormField>
      <PatientSexField idPrefix={idPrefix} register={register} errors={errors} />
      <PatientOptionalFields idPrefix={idPrefix} register={register} errors={errors} />
    </>
  );
}

function PatientSexField<T extends PatientFieldValues>({
  idPrefix,
  register,
  errors,
}: PatientFormFieldsProps<T>): JSX.Element {
  const fieldErrors = asPatientFieldErrors(errors);
  return (
    <FormField id={`${idPrefix}-sex`} label="Sex" error={fieldErrors.sex?.message}>
      <select id={`${idPrefix}-sex`} {...register('sex' as never)}>
        {SEX_OPTIONS.map((sex) => (
          <option key={sex} value={sex}>
            {sex}
          </option>
        ))}
      </select>
    </FormField>
  );
}

function PatientOptionalFields<T extends PatientFieldValues>({
  idPrefix,
  register,
  errors,
}: PatientFormFieldsProps<T>): JSX.Element {
  const fieldErrors = asPatientFieldErrors(errors);
  return (
    <>
      <FormField
        id={`${idPrefix}-national-id`}
        label="National ID"
        error={fieldErrors.nationalId?.message}
      >
        <input id={`${idPrefix}-national-id`} type="text" {...register('nationalId' as never)} />
      </FormField>
      <FormField id={`${idPrefix}-phone`} label="Phone" error={fieldErrors.phone?.message}>
        <input id={`${idPrefix}-phone`} type="text" {...register('phone' as never)} />
      </FormField>
      <FormField id={`${idPrefix}-address`} label="Address" error={fieldErrors.address?.message}>
        <input id={`${idPrefix}-address`} type="text" {...register('address' as never)} />
      </FormField>
      <PatientEmergencyContactFields idPrefix={idPrefix} register={register} errors={errors} />
      <FormField
        id={`${idPrefix}-blood-type`}
        label="Blood type"
        error={fieldErrors.bloodType?.message}
      >
        <input id={`${idPrefix}-blood-type`} type="text" {...register('bloodType' as never)} />
      </FormField>
      <FormField
        id={`${idPrefix}-allergies`}
        label="Allergies"
        error={fieldErrors.allergies?.message}
      >
        <textarea id={`${idPrefix}-allergies`} {...register('allergies' as never)} />
      </FormField>
    </>
  );
}

function PatientEmergencyContactFields<T extends PatientFieldValues>({
  idPrefix,
  register,
  errors,
}: PatientFormFieldsProps<T>): JSX.Element {
  const fieldErrors = asPatientFieldErrors(errors);
  return (
    <>
      <FormField
        id={`${idPrefix}-emergency-contact-name`}
        label="Emergency contact name"
        error={fieldErrors.emergencyContactName?.message}
      >
        <input
          id={`${idPrefix}-emergency-contact-name`}
          type="text"
          {...register('emergencyContactName' as never)}
        />
      </FormField>
      <FormField
        id={`${idPrefix}-emergency-contact-phone`}
        label="Emergency contact phone"
        error={fieldErrors.emergencyContactPhone?.message}
      >
        <input
          id={`${idPrefix}-emergency-contact-phone`}
          type="text"
          {...register('emergencyContactPhone' as never)}
        />
      </FormField>
    </>
  );
}
