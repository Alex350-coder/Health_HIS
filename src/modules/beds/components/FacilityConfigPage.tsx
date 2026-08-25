import { ErrorBoundary } from '@shared/components/ErrorBoundary';

import {
  CreateBedForm,
  CreateFloorForm,
  CreateRoomForm,
  PromoteRoomToOperatingRoomForm,
} from './FacilityConfigForms';
import { FacilityStructureView } from './FacilityStructureView';

/** The only write path for structural facility data — floors, rooms, beds (IPC.md Section 2.1). */
export default function FacilityConfigPage(): JSX.Element {
  return (
    <ErrorBoundary>
      <main className="flex flex-col gap-6 p-6">
        <h1 className="text-xl font-semibold text-text-primary">Facility Configuration</h1>
        <section className="grid grid-cols-1 gap-6 md:grid-cols-3">
          <div>
            <h2 className="text-base font-semibold text-text-primary">Add floor</h2>
            <CreateFloorForm />
          </div>
          <div>
            <h2 className="text-base font-semibold text-text-primary">Add room</h2>
            <CreateRoomForm />
          </div>
          <div>
            <h2 className="text-base font-semibold text-text-primary">Add bed</h2>
            <CreateBedForm />
          </div>
          <div>
            <h2 className="text-base font-semibold text-text-primary">Promote room to OR</h2>
            <PromoteRoomToOperatingRoomForm />
          </div>
        </section>
        <section>
          <h2 className="text-base font-semibold text-text-primary">Current layout</h2>
          <FacilityStructureView />
        </section>
      </main>
    </ErrorBoundary>
  );
}
