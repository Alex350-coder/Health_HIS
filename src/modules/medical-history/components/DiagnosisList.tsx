import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@shared/ui/Table';

import type { Diagnosis } from '../types/medical-history-schemas';

interface DiagnosisListProps {
  diagnoses: Diagnosis[];
}

/** Chronological read-only list — diagnoses are append-only, so there is nothing to edit here. */
export function DiagnosisList({ diagnoses }: DiagnosisListProps): JSX.Element {
  if (diagnoses.length === 0) {
    return <p className="text-sm text-text-secondary">No diagnoses recorded yet.</p>;
  }

  return (
    <Table aria-label="Diagnoses">
      <TableHeader>
        <TableRow>
          <TableHead>Description</TableHead>
          <TableHead>ICD code</TableHead>
          <TableHead>Recorded</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {diagnoses.map((diagnosis) => (
          <TableRow key={diagnosis.id}>
            <TableCell>
              {diagnosis.description}
              {diagnosis.correctsDiagnosisId !== null ? (
                <span className="ml-2 text-xs text-text-secondary">
                  (corrects #{diagnosis.correctsDiagnosisId})
                </span>
              ) : null}
            </TableCell>
            <TableCell>{diagnosis.icdCode ?? '—'}</TableCell>
            <TableCell>{new Date(diagnosis.createdAt).toLocaleString()}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
