import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { FormField } from '@shared/ui/FormField';

import { useCreateUser } from '../api/auth-mutations';
import { createUserSchema, type CreateUserInput } from '../types/auth-schemas';

const ROLE_OPTIONS = ['physician', 'nurse', 'admin', 'pharmacy', 'lab', 'receptionist'] as const;

export function CreateUserForm(): JSX.Element {
  const createUser = useCreateUser();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateUserInput>({ resolver: zodResolver(createUserSchema) });

  const onSubmit = handleSubmit((input) => {
    createUser.mutate(input, { onSuccess: () => reset() });
  });

  const errorMessage = formErrorMessage(createUser.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Create a user">
      <FormField id="create-user-full-name" label="Full name" error={errors.fullName?.message}>
        <input id="create-user-full-name" type="text" {...register('fullName')} />
      </FormField>
      <FormField id="create-user-username" label="Username" error={errors.username?.message}>
        <input id="create-user-username" type="text" {...register('username')} />
      </FormField>
      <FormField id="create-user-password" label="Password" error={errors.password?.message}>
        <input id="create-user-password" type="password" {...register('password')} />
      </FormField>
      <FormField id="create-user-role" label="Role" error={errors.role?.message}>
        <select id="create-user-role" {...register('role')}>
          {ROLE_OPTIONS.map((role) => (
            <option key={role} value={role}>
              {role}
            </option>
          ))}
        </select>
      </FormField>
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <button type="submit" disabled={createUser.isPending}>
        {createUser.isPending ? 'Creating…' : 'Create user'}
      </button>
    </form>
  );
}
