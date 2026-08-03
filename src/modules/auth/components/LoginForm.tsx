import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { FormField } from '@shared/components/FormField';
import { mutationErrorMessage } from '@shared/errors/error-messages';

import { useLogin } from '../api/auth-mutations';
import { useSessionStore } from '../hooks/use-session-store';
import { loginSchema, type LoginInput } from '../types/auth-schemas';

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

  const errorMessage = mutationErrorMessage(login.error);

  return (
    <form onSubmit={(event) => void onSubmit(event)} noValidate aria-label="Log in">
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
      {errorMessage ? <p role="alert">{errorMessage}</p> : null}
      <button type="submit" disabled={login.isPending}>
        {login.isPending ? 'Logging in…' : 'Log in'}
      </button>
    </form>
  );
}
