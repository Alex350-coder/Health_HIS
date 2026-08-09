import type { Patient } from '../types/patient-schemas';
import type { KeyboardEvent } from 'react';

interface PatientListRowProps {
  patient: Patient;
  onSelect: () => void;
}

/**
 * One row's cells for `VirtualizedTable`'s `renderRow` contract — the `<tr>` wrapper lives there.
 * Every cell shares the same click handler (a `<button>` can't wrap `<td>`s), and the first cell
 * is the row's single keyboard-focusable target.
 */
export function PatientListRow({ patient, onSelect }: PatientListRowProps): JSX.Element {
  const handleKeyDown = (event: KeyboardEvent): void => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onSelect();
    }
  };

  return (
    <>
      <td
        className="cursor-pointer p-3 align-middle"
        role="button"
        tabIndex={0}
        aria-label={`View patient ${patient.fullName}`}
        onClick={onSelect}
        onKeyDown={handleKeyDown}
      >
        {patient.medicalRecordNumber}
      </td>
      <td className="cursor-pointer p-3 align-middle" onClick={onSelect}>
        {patient.fullName}
      </td>
      <td className="cursor-pointer p-3 align-middle" onClick={onSelect}>
        {patient.dateOfBirth}
      </td>
      <td className="cursor-pointer p-3 align-middle capitalize" onClick={onSelect}>
        {patient.sex}
      </td>
    </>
  );
}
