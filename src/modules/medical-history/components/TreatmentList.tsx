import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@shared/ui/Table';

import type { Treatment } from '../types/medical-history-schemas';

interface TreatmentListProps {
  treatments: Treatment[];
}

/** Chronological read-only list — treatments are append-only, so there is nothing to edit here. */
export function TreatmentList({ treatments }: TreatmentListProps): JSX.Element {
  if (treatments.length === 0) {
    return <p className="text-sm text-text-secondary">No treatments recorded yet.</p>;
  }

  return (
    <Table aria-label="Treatments">
      <TableHeader>
        <TableRow>
          <TableHead>Description</TableHead>
          <TableHead>Dosage</TableHead>
          <TableHead>Recorded</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {treatments.map((treatment) => (
          <TableRow key={treatment.id}>
            <TableCell>
              {treatment.description}
              {treatment.correctsTreatmentId !== null ? (
                <span className="ml-2 text-xs text-text-secondary">
                  (corrects #{treatment.correctsTreatmentId})
                </span>
              ) : null}
            </TableCell>
            <TableCell>{treatment.dosage ?? '—'}</TableCell>
            <TableCell>{new Date(treatment.createdAt).toLocaleString()}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
