import * as LabelPrimitive from '@radix-ui/react-label';

import { cn } from '@shared/lib/cn';

type FormLabelProps = LabelPrimitive.LabelProps;

/** Always visible — placeholder-only labels are a Rule 19.1 violation (UI.md Section 8). */
export function FormLabel({ className, ...rest }: FormLabelProps): JSX.Element {
  return (
    <LabelPrimitive.Root
      className={cn('text-sm font-medium text-text-primary', className)}
      {...rest}
    />
  );
}
