import { zodResolver } from '@hookform/resolvers/zod';
import { useForm, type FieldErrors, type UseFormRegister } from 'react-hook-form';

import { AccountLockedDialog } from '@shared/components/AccountLockedDialog';
import { accountLockedRetrySecs, formErrorMessage } from '@shared/errors/error-messages';
import { FormField } from '@shared/ui/FormField';

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
        <input id="login-username" type="text" autoComplete="username" {...register('username')} />
      </FormField>
      <FormField id="login-password" label="Password" error={errors.password?.message}>
        <input
          id="login-password"
          type="password"
          autoComplete="current-password"
          {...register('password')}
        />
      </FormField>
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

  const onSubmit = handleSubmit((input) => {
    login.mutate(input, {
      onSuccess: (response) => {
        setSession(response.token, response.user);
      },
    });
  });

  const errorMessage = formErrorMessage(login.error);
  const retryAfterSecs = accountLockedRetrySecs(login.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Log in">
      <LoginFields register={register} errors={errors} />
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <button type="submit" disabled={login.isPending}>
        {login.isPending ? 'Logging in…' : 'Log in'}
      </button>
      <AccountLockedDialog
        key={retryAfterSecs ?? 'closed'}
        retryAfterSecs={retryAfterSecs}
        onOpenChange={() => login.reset()}
      />
    </form>
  );
}
