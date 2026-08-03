import { zodResolver } from '@hookform/resolvers/zod';
import { useForm, type FieldErrors, type UseFormRegister } from 'react-hook-form';

import { FormField } from '@shared/components/FormField';
import { mutationErrorMessage } from '@shared/errors/error-messages';

import { useBootstrapAdmin } from '../api/auth-mutations';
import { useSessionStore } from '../hooks/use-session-store';
import { createUserSchema, type CreateUserInput } from '../types/auth-schemas';

const FIELDS = [
  {
    name: 'fullName',
    id: 'bootstrap-full-name',
    label: 'Full name',
    type: 'text',
    autoComplete: 'name',
  },
  {
    name: 'username',
    id: 'bootstrap-username',
    label: 'Username',
    type: 'text',
    autoComplete: 'username',
  },
  {
    name: 'password',
    id: 'bootstrap-password',
    label: 'Password',
    type: 'password',
    autoComplete: 'new-password',
  },
] as const;

function BootstrapFields({
  register,
  errors,
}: {
  register: UseFormRegister<CreateUserInput>;
  errors: FieldErrors<CreateUserInput>;
}): JSX.Element {
  return (
    <>
      {FIELDS.map((field) => (
        <FormField
          key={field.id}
          id={field.id}
          label={field.label}
          error={errors[field.name]?.message}
        >
          <input
            id={field.id}
            type={field.type}
            autoComplete={field.autoComplete}
            {...register(field.name)}
          />
        </FormField>
      ))}
    </>
  );
}

/**
 * Only reachable when `auth_bootstrap_status` reports `needsBootstrap: true` — creating the
 * first `admin` user is the sole unauthenticated write path in the app (IPC.md Section 2.1).
 */
export function BootstrapForm(): JSX.Element {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<CreateUserInput>({
    resolver: zodResolver(createUserSchema),
    defaultValues: { role: 'admin' },
  });
  const bootstrapAdmin = useBootstrapAdmin();
  const setSession = useSessionStore((state) => state.setSession);

  const onSubmit = handleSubmit((input) => {
    bootstrapAdmin.mutate(input, {
      onSuccess: (response) => {
        setSession(response.token, response.user);
      },
    });
  });

  const errorMessage = mutationErrorMessage(bootstrapAdmin.error);

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Create the first administrator account"
    >
      <BootstrapFields register={register} errors={errors} />
      <input type="hidden" value="admin" {...register('role')} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <button type="submit" disabled={bootstrapAdmin.isPending}>
        {bootstrapAdmin.isPending ? 'Creating administrator…' : 'Create administrator account'}
      </button>
    </form>
  );
}
