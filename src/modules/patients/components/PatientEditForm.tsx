import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';

import { useUpdatePatient } from '../api/patient-mutations';
import {
  updatePatientSchema,
  type Patient,
  type UpdatePatientFormValues,
  type UpdatePatientInput,
} from '../types/patient-schemas';

import { PatientFormFields } from './PatientFormFields';

interface PatientEditFormProps {
  patient: Patient;
  onDone: () => void;
}

function toFormValues(patient: Patient): UpdatePatientFormValues {
  return {
    id: patient.id,
    medicalRecordNumber: patient.medicalRecordNumber,
    fullName: patient.fullName,
    dateOfBirth: patient.dateOfBirth,
    sex: patient.sex as UpdatePatientFormValues['sex'],
    nationalId: patient.nationalId ?? '',
    phone: patient.phone ?? '',
    address: patient.address ?? '',
    emergencyContactName: patient.emergencyContactName ?? '',
    emergencyContactPhone: patient.emergencyContactPhone ?? '',
    bloodType: patient.bloodType ?? '',
    allergies: patient.allergies ?? '',
  };
}

export function PatientEditForm({ patient, onDone }: PatientEditFormProps): JSX.Element {
  const updatePatient = useUpdatePatient();
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<UpdatePatientFormValues, unknown, UpdatePatientInput>({
    resolver: zodResolver(updatePatientSchema),
    defaultValues: toFormValues(patient),
  });

  const onSubmit = handleSubmit((input) => {
    updatePatient.mutate(input, { onSuccess: onDone });
  });

  const errorMessage = formErrorMessage(updatePatient.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Edit patient">
      <PatientFormFields idPrefix="edit-patient" register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <div className="flex gap-2">
        <button type="submit" disabled={updatePatient.isPending}>
          {updatePatient.isPending ? 'Saving…' : 'Save changes'}
        </button>
        <button type="button" onClick={onDone} disabled={updatePatient.isPending}>
          Cancel
        </button>
      </div>
    </form>
  );
}
