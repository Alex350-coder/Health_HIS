import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from '@tanstack/react-router';
import { useForm, type FieldErrors, type UseFormRegister } from 'react-hook-form';

import { AccountLockedDialog } from '@shared/components/AccountLockedDialog';
import { accountLockedRetrySecs, formErrorMessage } from '@shared/errors/error-messages';
import { Button } from '@shared/ui/Button';
import { FormField } from '@shared/ui/FormField';
import { Input } from '@shared/ui/Input';

import { useLogin } from '../api/auth-mutations';
import { useSessionStore } from '../hooks/use-session-store';
import { loginSchema, type LoginInput } from '../types/auth-schemas';

function LoginFields({
  register,
  errors,
}: {
  register: UseFormRegister<LoginInput>;
  errors: FieldErrors<LoginInput>;
}): JSX.Element {
  return (
    <>
      <FormField id="login-username" label="Username" error={errors.username?.message}>
        <Input
          id="login-username"
          type="text"
          autoComplete="username"
          aria-invalid={errors.username !== undefined}
          {...register('username')}
        />
      </FormField>
      <FormField id="login-password" label="Password" error={errors.password?.message}>
        <Input
          id="login-password"
          type="password"
          autoComplete="current-password"
          aria-invalid={errors.password !== undefined}
          {...register('password')}
        />
      </FormField>
    </>
  );
}

function LoginStatus({ login }: { login: ReturnType<typeof useLogin> }): JSX.Element {
  const errorMessage = formErrorMessage(login.error);
  const retryAfterSecs = accountLockedRetrySecs(login.error);

  return (
    <>
      {errorMessage ? (
        <p role="alert" className="text-sm text-danger">
          {errorMessage}
        </p>
      ) : null}
      <Button type="submit" disabled={login.isPending} className="w-full">
        {login.isPending ? 'Logging in…' : 'Log in'}
      </Button>
      <AccountLockedDialog
        key={retryAfterSecs ?? 'closed'}
        retryAfterSecs={retryAfterSecs}
        onOpenChange={() => login.reset()}
      />
    </>
  );
}

export function LoginForm(): JSX.Element {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginInput>({ resolver: zodResolver(loginSchema) });
  const login = useLogin();
  const setSession = useSessionStore((state) => state.setSession);
  const navigate = useNavigate();

  const onSubmit = handleSubmit((input) => {
    login.mutate(input, {
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
      aria-label="Log in"
      className="flex flex-col gap-4"
    >
      <LoginFields register={register} errors={errors} />
      <LoginStatus login={login} />
    </form>
  );
}
