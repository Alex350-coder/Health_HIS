import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';

import { useDischargeEncounter } from '../api/medical-history-mutations';
import {
  dischargeEncounterSchema,
  type DischargeEncounterFormValues,
  type DischargeEncounterInput,
} from '../types/medical-history-schemas';

interface DischargeFormProps {
  encounterId: number;
  onDone: () => void;
}

/** Confirms discharge with an optional summary; sets the encounter to `discharged`. */
export function DischargeForm({ encounterId, onDone }: DischargeFormProps): JSX.Element {
  const dischargeEncounter = useDischargeEncounter();
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<DischargeEncounterFormValues, unknown, DischargeEncounterInput>({
    resolver: zodResolver(dischargeEncounterSchema),
    defaultValues: { encounterId },
  });

  const onSubmit = handleSubmit((input) => {
    dischargeEncounter.mutate(input, { onSuccess: onDone });
  });

  const errorMessage = formErrorMessage(dischargeEncounter.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Discharge encounter"
      className="flex flex-col gap-3"
    >
      <FormField
        id="discharge-summary"
        label="Discharge summary"
        error={errors.dischargeSummary?.message}
      >
        <textarea id="discharge-summary" {...register('dischargeSummary')} />
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <div className="flex gap-2">
        <Button type="submit" intent="danger" size="sm" disabled={dischargeEncounter.isPending}>
          {dischargeEncounter.isPending ? 'Discharging…' : 'Confirm discharge'}
        </Button>
        <Button type="button" intent="ghost" size="sm" onClick={onDone}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
