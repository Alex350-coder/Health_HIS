import type { ReactNode } from 'react';

interface FormFieldProps {
  id: string;
  label: string;
  error?: string | undefined;
  children: ReactNode;
}

/** Label + input + inline validation error, per UI.md Section 8 (every input has a visible label). */
export function FormField({ id, label, error, children }: FormFieldProps): JSX.Element {
  return (
    <div>
      <label htmlFor={id}>{label}</label>
      {children}
      {error ? <p role="alert">{error}</p> : null}
    </div>
  );
}
