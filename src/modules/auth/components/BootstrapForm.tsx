import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from '@tanstack/react-router';
import { useForm, type FieldErrors, type UseFormRegister } from 'react-hook-form';

import { formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';
import { Input } from '@shared/ui/Input';

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
          <Input
            id={field.id}
            type={field.type}
            autoComplete={field.autoComplete}
            aria-invalid={errors[field.name] !== undefined}
            {...register(field.name)}
          />
        </FormField>
      ))}
    </>
  );
}

function BootstrapStatus({
  bootstrapAdmin,
}: {
  bootstrapAdmin: ReturnType<typeof useBootstrapAdmin>;
}): JSX.Element {
  const errorMessage = formErrorMessage(bootstrapAdmin.error);

  return (
    <>
      {errorMessage ? (
        <p role="alert" className="text-sm text-danger">
          {errorMessage}
        </p>
      ) : null}
      <Button type="submit" disabled={bootstrapAdmin.isPending} className="w-full">
        {bootstrapAdmin.isPending ? 'Creating administrator…' : 'Create administrator account'}
      </Button>
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
  const navigate = useNavigate();

  const onSubmit = handleSubmit((input) => {
    bootstrapAdmin.mutate(input, {
      onSuccess: (response) => {
        setSession(response.token, response.user);
        void navigate({ to: '/' });
      },
    });
  });

  return (
    <form
      onSubmit={(event) => void onSubmit(event)}
      noValidate
      aria-label="Create the first administrator account"
      className="flex flex-col gap-4"
    >
      <BootstrapFields register={register} errors={errors} />
      <input type="hidden" value="admin" {...register('role')} />
      <BootstrapStatus bootstrapAdmin={bootstrapAdmin} />
    </form>
  );
}
