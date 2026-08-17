import type { Evolution } from '../types/medical-history-schemas';

interface EvolutionListProps {
  evolutions: Evolution[];
}

/** Chronological read-only list — evolutions are append-only, so there is nothing to edit here. */
export function EvolutionList({ evolutions }: EvolutionListProps): JSX.Element {
  if (evolutions.length === 0) {
    return <p className="text-sm text-text-secondary">No evolution notes recorded yet.</p>;
  }

  return (
    <ul className="flex flex-col gap-3">
      {evolutions.map((evolution) => (
        <li key={evolution.id} className="rounded-md border border-border-default p-3">
          <p className="text-sm text-text-primary">{evolution.note}</p>
          <p className="mt-1 text-xs text-text-secondary">
            {new Date(evolution.createdAt).toLocaleString()}
          </p>
        </li>
      ))}
    </ul>
  );
}
