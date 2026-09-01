import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { Dialog, DialogContent, DialogFooter, DialogTrigger } from '@shared/ui/Dialog';
import { FormField } from '@shared/ui/FormField';

import { useRecordInventoryTransaction } from '../api/inventory-mutations';
import {
  createInventoryTransactionSchema,
  INVENTORY_TRANSACTION_REASONS,
  type CreateInventoryTransactionFormValues,
  type CreateInventoryTransactionInput,
} from '../types/inventory-schemas';

import type { FieldErrors, UseFormRegister } from 'react-hook-form';

interface RecordInventoryTransactionDialogProps {
  itemId: number;
}

interface RecordInventoryTransactionFieldsProps {
  register: UseFormRegister<CreateInventoryTransactionFormValues>;
  errors: FieldErrors<CreateInventoryTransactionFormValues>;
}

function RecordInventoryTransactionFields({
  register,
  errors,
}: RecordInventoryTransactionFieldsProps): JSX.Element {
  return (
    <>
      <FormField
        id="inventory-transaction-quantity-delta"
        label="Quantity change"
        error={errors.quantityDelta?.message}
      >
        <input
          id="inventory-transaction-quantity-delta"
          type="number"
          {...register('quantityDelta')}
        />
      </FormField>
      <FormField id="inventory-transaction-reason" label="Reason" error={errors.reason?.message}>
        <select id="inventory-transaction-reason" {...register('reason')}>
          {INVENTORY_TRANSACTION_REASONS.map((reason) => (
            <option key={reason} value={reason}>
              {reason}
            </option>
          ))}
        </select>
      </FormField>
    </>
  );
}

/** Offered from the item detail page to record a stock movement (restock, consumption, etc.). */
export function RecordInventoryTransactionDialog({
  itemId,
}: RecordInventoryTransactionDialogProps): JSX.Element {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button size="sm">Record transaction</Button>
      </DialogTrigger>
      <DialogContent
        title="Record inventory transaction"
        description="Adjust this item's stock and record why."
      >
        <RecordInventoryTransactionForm itemId={itemId} />
      </DialogContent>
    </Dialog>
  );
}

function RecordInventoryTransactionForm({
  itemId,
}: RecordInventoryTransactionDialogProps): JSX.Element {
  const recordTransaction = useRecordInventoryTransaction();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateInventoryTransactionFormValues, unknown, CreateInventoryTransactionInput>({
    resolver: zodResolver(createInventoryTransactionSchema),
    defaultValues: { itemId, reason: 'restock' },
  });

  const onSubmit = handleSubmit((input) => {
    recordTransaction.mutate(input, { onSuccess: () => reset({ itemId, reason: 'restock' }) });
  });

  const errorMessage = formErrorMessage(recordTransaction.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Record inventory transaction"
    >
      <input type="hidden" {...register('itemId')} />
      <RecordInventoryTransactionFields register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <DialogFooter>
        <Button type="submit" disabled={recordTransaction.isPending}>
          {recordTransaction.isPending ? 'Recording…' : 'Record'}
        </Button>
      </DialogFooter>
    </form>
  );
}
