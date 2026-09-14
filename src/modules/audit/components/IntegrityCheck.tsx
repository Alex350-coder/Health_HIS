import { Button } from '@shared/ui/Button';

import { useVerifyIntegrity } from '../api/audit-mutations';

export function IntegrityCheck(): JSX.Element {
  const verifyIntegrity = useVerifyIntegrity();

  return (
    <div className="flex flex-col items-start gap-2">
      <Button
        type="button"
        intent="secondary"
        size="sm"
        onClick={() => verifyIntegrity.mutate()}
        disabled={verifyIntegrity.isPending}
      >
        {verifyIntegrity.isPending ? 'Verifying…' : 'Verify integrity'}
      </Button>
      {verifyIntegrity.isSuccess ? (
        <p role="status" className="text-sm text-text-secondary">
          {verifyIntegrity.data.isValid
            ? 'The audit trail is intact.'
            : `Tampering detected at audit entry ${String(verifyIntegrity.data.brokenAtId)}.`}
        </p>
      ) : null}
      {verifyIntegrity.isError ? (
        <p role="alert" className="text-sm text-danger">
          Could not verify the audit trail.
        </p>
      ) : null}
    </div>
  );
}
