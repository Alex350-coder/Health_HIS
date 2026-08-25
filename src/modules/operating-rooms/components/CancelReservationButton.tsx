import { Button } from '@shared/ui/Button';

import { useCancelOrReservation } from '../api/or-mutations';

interface CancelReservationButtonProps {
  reservationId: number;
}

export function CancelReservationButton({
  reservationId,
}: CancelReservationButtonProps): JSX.Element {
  const cancelReservation = useCancelOrReservation();

  return (
    <Button
      size="sm"
      intent="secondary"
      disabled={cancelReservation.isPending}
      onClick={() => cancelReservation.mutate({ id: reservationId })}
    >
      {cancelReservation.isPending ? 'Cancelling…' : 'Cancel'}
    </Button>
  );
}
