import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';

import { useCreateInventoryCategory } from '../api/inventory-mutations';
import {
  createInventoryCategorySchema,
  INVENTORY_CATEGORY_KINDS,
  type CreateInventoryCategoryFormValues,
  type CreateInventoryCategoryInput,
} from '../types/inventory-schemas';

/** Inline add-category form — this module owns its own tables, so it lives on its own list page. */
export function CreateInventoryCategoryForm(): JSX.Element {
  const createCategory = useCreateInventoryCategory();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateInventoryCategoryFormValues, unknown, CreateInventoryCategoryInput>({
    resolver: zodResolver(createInventoryCategorySchema),
  });

  const onSubmit = handleSubmit((input) => {
    createCategory.mutate(input, { onSuccess: () => reset() });
  });

  const errorMessage = formErrorMessage(createCategory.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Add inventory category">
      <FormField id="inventory-category-name" label="Category name" error={errors.name?.message}>
        <input id="inventory-category-name" type="text" {...register('name')} />
      </FormField>
      <FormField id="inventory-category-kind" label="Kind" error={errors.kind?.message}>
        <select id="inventory-category-kind" {...register('kind')}>
          {INVENTORY_CATEGORY_KINDS.map((kind) => (
            <option key={kind} value={kind}>
              {kind}
            </option>
          ))}
        </select>
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <Button type="submit" disabled={createCategory.isPending}>
        {createCategory.isPending ? 'Adding…' : 'Add category'}
      </Button>
    </form>
  );
}
