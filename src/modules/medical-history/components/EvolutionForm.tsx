import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';

import { useCreateEvolution } from '../api/medical-history-mutations';
import {
  createEvolutionSchema,
  type CreateEvolutionFormValues,
  type CreateEvolutionInput,
} from '../types/medical-history-schemas';

interface EvolutionFormProps {
  encounterId: number;
}

/** Inline add-evolution-note form for the open encounter. Plain append, no correction concept. */
export function EvolutionForm({ encounterId }: EvolutionFormProps): JSX.Element {
  const createEvolution = useCreateEvolution();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateEvolutionFormValues, unknown, CreateEvolutionInput>({
    resolver: zodResolver(createEvolutionSchema),
    defaultValues: { encounterId },
  });

  const onSubmit = handleSubmit((input) => {
    createEvolution.mutate(input, { onSuccess: () => reset({ encounterId }) });
  });

  const errorMessage = formErrorMessage(createEvolution.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Add evolution note"
      className="flex flex-col gap-3"
    >
      <FormField id="evolution-note" label="Note" error={errors.note?.message}>
        <textarea id="evolution-note" {...register('note')} />
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" size="sm" disabled={createEvolution.isPending}>
        {createEvolution.isPending ? 'Adding…' : 'Add note'}
      </Button>
    </form>
  );
}
