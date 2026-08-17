import { zodResolver } from '@hookform/resolvers/zod';
import { useForm, type FieldErrors, type UseFormRegister } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';

import { useCreateTreatment } from '../api/medical-history-mutations';
import {
  createTreatmentSchema,
  type CreateTreatmentFormValues,
  type CreateTreatmentInput,
  type Diagnosis,
  type Treatment,
} from '../types/medical-history-schemas';

interface TreatmentFormProps {
  encounterId: number;
  existingDiagnoses: Diagnosis[];
  existingTreatments: Treatment[];
}

/** Inline add-treatment form for the open encounter. Corrections reference a prior treatment id. */
export function TreatmentForm({
  encounterId,
  existingDiagnoses,
  existingTreatments,
}: TreatmentFormProps): JSX.Element {
  const createTreatment = useCreateTreatment();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateTreatmentFormValues, unknown, CreateTreatmentInput>({
    resolver: zodResolver(createTreatmentSchema),
    defaultValues: { encounterId },
  });

  const onSubmit = handleSubmit((input) => {
    createTreatment.mutate(input, { onSuccess: () => reset({ encounterId }) });
  });

  const errorMessage = formErrorMessage(createTreatment.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Add treatment"
      className="flex flex-col gap-3"
    >
      <TreatmentFormFields
        register={register}
        errors={errors}
        existingDiagnoses={existingDiagnoses}
        existingTreatments={existingTreatments}
      />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" size="sm" disabled={createTreatment.isPending}>
        {createTreatment.isPending ? 'Adding…' : 'Add treatment'}
      </Button>
    </form>
  );
}

function TreatmentFormFields({
  register,
  errors,
  existingDiagnoses,
  existingTreatments,
}: {
  register: UseFormRegister<CreateTreatmentFormValues>;
  errors: FieldErrors<CreateTreatmentFormValues>;
  existingDiagnoses: Diagnosis[];
  existingTreatments: Treatment[];
}): JSX.Element {
  return (
    <>
      <FormField id="treatment-description" label="Description" error={errors.description?.message}>
        <textarea id="treatment-description" {...register('description')} />
      </FormField>
      <FormField id="treatment-dosage" label="Dosage" error={errors.dosage?.message}>
        <input id="treatment-dosage" type="text" {...register('dosage')} />
      </FormField>
      <TreatmentFormRelationFields
        register={register}
        errors={errors}
        existingDiagnoses={existingDiagnoses}
        existingTreatments={existingTreatments}
      />
    </>
  );
}

function TreatmentFormRelationFields({
  register,
  errors,
  existingDiagnoses,
  existingTreatments,
}: {
  register: UseFormRegister<CreateTreatmentFormValues>;
  errors: FieldErrors<CreateTreatmentFormValues>;
  existingDiagnoses: Diagnosis[];
  existingTreatments: Treatment[];
}): JSX.Element {
  return (
    <>
      <TreatmentDiagnosisField
        register={register}
        errors={errors}
        existingDiagnoses={existingDiagnoses}
      />
      <TreatmentCorrectsField
        register={register}
        errors={errors}
        existingTreatments={existingTreatments}
      />
    </>
  );
}

function TreatmentDiagnosisField({
  register,
  errors,
  existingDiagnoses,
}: {
  register: UseFormRegister<CreateTreatmentFormValues>;
  errors: FieldErrors<CreateTreatmentFormValues>;
  existingDiagnoses: Diagnosis[];
}): JSX.Element | null {
  if (existingDiagnoses.length === 0) {
    return null;
  }
  return (
    <FormField
      id="treatment-diagnosis"
      label="Related diagnosis"
      error={errors.diagnosisId?.message}
    >
      <select id="treatment-diagnosis" {...register('diagnosisId', { valueAsNumber: true })}>
        <option value="">None</option>
        {existingDiagnoses.map((diagnosis) => (
          <option key={diagnosis.id} value={diagnosis.id}>
            #{diagnosis.id} — {diagnosis.description}
          </option>
        ))}
      </select>
    </FormField>
  );
}

function TreatmentCorrectsField({
  register,
  errors,
  existingTreatments,
}: {
  register: UseFormRegister<CreateTreatmentFormValues>;
  errors: FieldErrors<CreateTreatmentFormValues>;
  existingTreatments: Treatment[];
}): JSX.Element | null {
  if (existingTreatments.length === 0) {
    return null;
  }
  return (
    <FormField
      id="treatment-corrects"
      label="Corrects treatment"
      error={errors.correctsTreatmentId?.message}
    >
      <select id="treatment-corrects" {...register('correctsTreatmentId', { valueAsNumber: true })}>
        <option value="">None</option>
        {existingTreatments.map((treatment) => (
          <option key={treatment.id} value={treatment.id}>
            #{treatment.id} — {treatment.description}
          </option>
        ))}
      </select>
    </FormField>
  );
}
