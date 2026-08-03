import { useVerifyIntegrity } from '../api/audit-mutations';

export function IntegrityCheck(): JSX.Element {
  const verifyIntegrity = useVerifyIntegrity();

  return (
    <div>
      <button
        type="button"
        onClick={() => verifyIntegrity.mutate()}
        disabled={verifyIntegrity.isPending}
      >
        {verifyIntegrity.isPending ? 'Verifying…' : 'Verify integrity'}
      </button>
      {verifyIntegrity.isSuccess ? (
        <p role="status">
          {verifyIntegrity.data.isValid
            ? 'The audit trail is intact.'
            : `Tampering detected at audit entry ${String(verifyIntegrity.data.brokenAtId)}.`}
        </p>
      ) : null}
      {verifyIntegrity.isError ? <p role="alert">Could not verify the audit trail.</p> : null}
    </div>
  );
}
