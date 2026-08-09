import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from '@tanstack/react-router';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';

import { useCreatePatient } from '../api/patient-mutations';
import {
  createPatientSchema,
  type CreatePatientFormValues,
  type CreatePatientInput,
} from '../types/patient-schemas';

import { PatientFormFields } from './PatientFormFields';

export function PatientCreateForm(): JSX.Element {
  const createPatient = useCreatePatient();
  const navigate = useNavigate();
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<CreatePatientFormValues, unknown, CreatePatientInput>({
    resolver: zodResolver(createPatientSchema),
  });

  const onSubmit = handleSubmit((input) => {
    createPatient.mutate(input, {
      onSuccess: (patient) => {
        void navigate({ to: `/patients/${String(patient.id)}` });
      },
    });
  });

  const errorMessage = formErrorMessage(createPatient.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Register patient">
      <PatientFormFields idPrefix="create-patient" register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <button type="submit" disabled={createPatient.isPending}>
        {createPatient.isPending ? 'Registering…' : 'Register patient'}
      </button>
    </form>
  );
}
