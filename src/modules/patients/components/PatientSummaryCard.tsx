import { useState } from 'react';

import { Button } from '@shared/ui/Button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@shared/ui/Card';

import { PatientEditForm } from './PatientEditForm';

import type { Patient } from '../types/patient-schemas';

interface PatientSummaryCardProps {
  patient: Patient;
}

const DISPLAY_FIELDS: readonly { label: string; value: (patient: Patient) => string }[] = [
  { label: 'Medical record number', value: (patient) => patient.medicalRecordNumber },
  { label: 'Date of birth', value: (patient) => patient.dateOfBirth },
  { label: 'Sex', value: (patient) => patient.sex },
  { label: 'National ID', value: (patient) => patient.nationalId ?? '—' },
  { label: 'Phone', value: (patient) => patient.phone ?? '—' },
  { label: 'Address', value: (patient) => patient.address ?? '—' },
  { label: 'Emergency contact name', value: (patient) => patient.emergencyContactName ?? '—' },
  { label: 'Emergency contact phone', value: (patient) => patient.emergencyContactPhone ?? '—' },
  { label: 'Blood type', value: (patient) => patient.bloodType ?? '—' },
  { label: 'Allergies', value: (patient) => patient.allergies ?? '—' },
];

/** Presentational display of a patient with an inline edit toggle (Decision 1 in the phase plan). */
export function PatientSummaryCard({ patient }: PatientSummaryCardProps): JSX.Element {
  const [isEditing, setIsEditing] = useState(false);

  if (isEditing) {
    return <PatientEditForm patient={patient} onDone={() => setIsEditing(false)} />;
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{patient.fullName}</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-2 gap-4">
          {DISPLAY_FIELDS.map((field) => (
            <div key={field.label}>
              <dt className="text-sm text-text-secondary">{field.label}</dt>
              <dd className="text-sm text-text-primary">{field.value(patient)}</dd>
            </div>
          ))}
        </dl>
      </CardContent>
      <CardFooter>
        <Button intent="secondary" size="sm" onClick={() => setIsEditing(true)}>
          Edit
        </Button>
      </CardFooter>
    </Card>
  );
}
