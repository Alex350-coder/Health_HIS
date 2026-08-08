import { FormError } from './FormError';
import { FormLabel } from './FormLabel';

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
    <div className="flex flex-col gap-1.5">
      <FormLabel htmlFor={id}>{label}</FormLabel>
      {children}
      <FormError>{error}</FormError>
    </div>
  );
}
