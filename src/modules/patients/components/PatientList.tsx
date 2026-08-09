import { useNavigate } from '@tanstack/react-router';
import { useEffect, useState } from 'react';

import { EmptyState } from '@shared/ui/EmptyState';
import { ErrorState } from '@shared/ui/ErrorState';
import { Skeleton } from '@shared/ui/Skeleton';
import { VirtualizedTable } from '@shared/ui/VirtualizedTable';

import { usePatientsList } from '../api/patient-queries';

import { PatientListRow } from './PatientListRow';

import type { Patient } from '../types/patient-schemas';
import type { UseQueryResult } from '@tanstack/react-query';

const PAGE_SIZE = 50;
const SEARCH_DEBOUNCE_MS = 300;

function useDebouncedValue(value: string, delayMs: number): string {
  const [debounced, setDebounced] = useState(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(timer);
  }, [value, delayMs]);

  return debounced;
}

function renderHeader(): JSX.Element {
  return (
    <tr>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">MRN</th>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Full name</th>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Date of birth</th>
      <th className="p-3 text-left text-sm font-semibold text-text-secondary">Sex</th>
    </tr>
  );
}

function PatientListResults({ query }: { query: UseQueryResult<Patient[]> }): JSX.Element {
  const navigate = useNavigate();

  if (query.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }
  if (query.isError) {
    return (
      <ErrorState
        description="Could not load the patient list."
        onRetry={() => void query.refetch()}
      />
    );
  }
  if (!query.isSuccess || query.data.length === 0) {
    return <EmptyState title="No patients found" description="Try a different search term." />;
  }
  return (
    <VirtualizedTable<Patient>
      items={query.data}
      getRowKey={(patient) => patient.id}
      renderHeader={renderHeader}
      renderRow={(patient) => (
        <PatientListRow
          patient={patient}
          onSelect={() => void navigate({ to: `/patients/${String(patient.id)}` })}
        />
      )}
    />
  );
}

/** Container: debounced search + `usePatientsList` + the mandatory loading/empty/error triad. */
export function PatientList(): JSX.Element {
  const [search, setSearch] = useState('');
  const debouncedSearch = useDebouncedValue(search, SEARCH_DEBOUNCE_MS);

  const query = usePatientsList(
    debouncedSearch
      ? { search: debouncedSearch, limit: PAGE_SIZE, offset: 0 }
      : { limit: PAGE_SIZE, offset: 0 },
    true,
  );

  return (
    <div className="flex flex-col gap-4">
      <input
        type="search"
        aria-label="Search patients"
        placeholder="Search by name or medical record number…"
        value={search}
        onChange={(event) => setSearch(event.target.value)}
        className="h-10 rounded-md border border-border-default px-3 text-sm"
      />
      <PatientListResults query={query} />
    </div>
  );
}
