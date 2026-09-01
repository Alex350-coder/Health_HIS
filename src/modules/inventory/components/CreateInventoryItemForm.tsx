import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';

import { useCreateInventoryItem } from '../api/inventory-mutations';
import {
  createInventoryItemSchema,
  type CreateInventoryItemFormValues,
  type CreateInventoryItemInput,
  type InventoryCategory,
} from '../types/inventory-schemas';

import type { FieldErrors, UseFormRegister } from 'react-hook-form';

interface CreateInventoryItemFormProps {
  categories: InventoryCategory[];
}

function CreateInventoryItemIdentityFields({
  register,
  errors,
  categories,
}: {
  register: UseFormRegister<CreateInventoryItemFormValues>;
  errors: FieldErrors<CreateInventoryItemFormValues>;
  categories: InventoryCategory[];
}): JSX.Element {
  return (
    <>
      <FormField id="inventory-item-category" label="Category" error={errors.categoryId?.message}>
        <select id="inventory-item-category" {...register('categoryId')}>
          <option value="">Select a category…</option>
          {categories.map((category) => (
            <option key={category.id} value={category.id}>
              {category.name}
            </option>
          ))}
        </select>
      </FormField>
      <FormField id="inventory-item-name" label="Name" error={errors.name?.message}>
        <input id="inventory-item-name" type="text" {...register('name')} />
      </FormField>
      <FormField id="inventory-item-unit" label="Unit" error={errors.unit?.message}>
        <input id="inventory-item-unit" type="text" {...register('unit')} />
      </FormField>
    </>
  );
}

function CreateInventoryItemStockFields({
  register,
  errors,
}: {
  register: UseFormRegister<CreateInventoryItemFormValues>;
  errors: FieldErrors<CreateInventoryItemFormValues>;
}): JSX.Element {
  return (
    <>
      <FormField
        id="inventory-item-reorder-threshold"
        label="Reorder threshold"
        error={errors.reorderThreshold?.message}
      >
        <input
          id="inventory-item-reorder-threshold"
          type="number"
          min={0}
          {...register('reorderThreshold')}
        />
      </FormField>
      <FormField
        id="inventory-item-expiration-date"
        label="Expiration date"
        error={errors.expirationDate?.message}
      >
        <input id="inventory-item-expiration-date" type="date" {...register('expirationDate')} />
      </FormField>
      <FormField id="inventory-item-location" label="Location" error={errors.location?.message}>
        <input id="inventory-item-location" type="text" {...register('location')} />
      </FormField>
    </>
  );
}

/** Inline add-item form — only offered once at least one category exists. */
export function CreateInventoryItemForm({ categories }: CreateInventoryItemFormProps): JSX.Element {
  const createItem = useCreateInventoryItem();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateInventoryItemFormValues, unknown, CreateInventoryItemInput>({
    resolver: zodResolver(createInventoryItemSchema),
  });

  const onSubmit = handleSubmit((input) => {
    createItem.mutate(input, { onSuccess: () => reset() });
  });

  const errorMessage = formErrorMessage(createItem.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Add inventory item">
      <CreateInventoryItemIdentityFields
        register={register}
        errors={errors}
        categories={categories}
      />
      <CreateInventoryItemStockFields register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" disabled={createItem.isPending}>
        {createItem.isPending ? 'Adding…' : 'Add item'}
      </Button>
    </form>
  );
}
