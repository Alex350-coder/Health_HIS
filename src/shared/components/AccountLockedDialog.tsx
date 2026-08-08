import { useEffect, useState } from 'react';

import { Dialog, DialogContent } from '@shared/ui/Dialog';

interface AccountLockedDialogProps {
  retryAfterSecs: number | undefined;
  onOpenChange: (open: boolean) => void;
}

function useCountdown(retryAfterSecs: number | undefined): number {
  const [secondsLeft, setSecondsLeft] = useState(retryAfterSecs ?? 0);

  useEffect(() => {
    if (retryAfterSecs === undefined) {
      return undefined;
    }
    const intervalId = setInterval(() => {
      setSecondsLeft((current) => Math.max(0, current - 1));
    }, 1000);
    return () => clearInterval(intervalId);
  }, [retryAfterSecs]);

  return secondsLeft;
}

/**
 * Countdown modal for `AppError::AccountLocked` (ErrorHandling.md Section 3). Controlled: the
 * caller passes `retryAfterSecs` from the failed mutation's error and clears it (setting this
 * prop to `undefined`) to close.
 */
export function AccountLockedDialog({
  retryAfterSecs,
  onOpenChange,
}: AccountLockedDialogProps): JSX.Element {
  const secondsLeft = useCountdown(retryAfterSecs);

  return (
    <Dialog open={retryAfterSecs !== undefined} onOpenChange={onOpenChange}>
      <DialogContent title="Account temporarily locked">
        {secondsLeft > 0 ? (
          <p>
            Too many failed attempts. Try again in{' '}
            <span className="font-semibold tabular-nums">{secondsLeft}</span> second
            {secondsLeft === 1 ? '' : 's'}.
          </p>
        ) : (
          <p>You can try again now.</p>
        )}
      </DialogContent>
    </Dialog>
  );
}
