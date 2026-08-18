import { zodResolver } from '@hookform/resolvers/zod';
import { useForm, type FieldErrors, type UseFormRegister } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';

import { useCreateDiagnosis } from '../api/medical-history-mutations';
import {
  createDiagnosisSchema,
  type CreateDiagnosisFormValues,
  type CreateDiagnosisInput,
  type Diagnosis,
} from '../types/medical-history-schemas';

interface DiagnosisFormProps {
  encounterId: number;
  existingDiagnoses: Diagnosis[];
}

/** Inline add-diagnosis form for the open encounter. Corrections reference a prior diagnosis id. */
export function DiagnosisForm({ encounterId, existingDiagnoses }: DiagnosisFormProps): JSX.Element {
  const createDiagnosis = useCreateDiagnosis();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateDiagnosisFormValues, unknown, CreateDiagnosisInput>({
    resolver: zodResolver(createDiagnosisSchema),
    defaultValues: { encounterId },
  });

  const onSubmit = handleSubmit((input) => {
    createDiagnosis.mutate(input, { onSuccess: () => reset({ encounterId }) });
  });

  const errorMessage = formErrorMessage(createDiagnosis.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Add diagnosis"
      className="flex flex-col gap-3"
    >
      <DiagnosisFormFields
        register={register}
        errors={errors}
        existingDiagnoses={existingDiagnoses}
      />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" size="sm" disabled={createDiagnosis.isPending}>
        {createDiagnosis.isPending ? 'Adding…' : 'Add diagnosis'}
      </Button>
    </form>
  );
}

function DiagnosisFormFields({
  register,
  errors,
  existingDiagnoses,
}: {
  register: UseFormRegister<CreateDiagnosisFormValues>;
  errors: FieldErrors<CreateDiagnosisFormValues>;
  existingDiagnoses: Diagnosis[];
}): JSX.Element {
  return (
    <>
      <FormField id="diagnosis-description" label="Description" error={errors.description?.message}>
        <textarea id="diagnosis-description" {...register('description')} />
      </FormField>
      <FormField id="diagnosis-icd-code" label="ICD code" error={errors.icdCode?.message}>
        <input id="diagnosis-icd-code" type="text" {...register('icdCode')} />
      </FormField>
      {existingDiagnoses.length > 0 ? (
        <FormField
          id="diagnosis-corrects"
          label="Corrects diagnosis"
          error={errors.correctsDiagnosisId?.message}
        >
          <select
            id="diagnosis-corrects"
            {...register('correctsDiagnosisId', { valueAsNumber: true })}
          >
            <option value="">None</option>
            {existingDiagnoses.map((diagnosis) => (
              <option key={diagnosis.id} value={diagnosis.id}>
                #{diagnosis.id} — {diagnosis.description}
              </option>
            ))}
          </select>
        </FormField>
      ) : null}
    </>
  );
}
