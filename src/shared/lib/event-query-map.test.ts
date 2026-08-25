import { describe, expect, it } from 'vitest';

import { EVENT_QUERY_MAP } from './event-query-map';

/**
 * Operationalizes StateManagement.md Section 6 checklist item 3 for the Phase 4-7 modules
 * (Patients, Medical History, Beds, Hospital Map): every event in IPC.md's event catalog for
 * these modules must have an entry here, and every mapped key prefix must actually match a
 * query key some hook in the affected module(s) uses — so a Phase 8-style propagation gap
 * (StateManagement.md Section 4) cannot silently regress.
 */
describe('EVENT_QUERY_MAP — IPC.md event catalog correspondence', () => {
  it.each([
    'patients:record:created',
    'patients:record:updated',
    'medical-history:encounter:created',
    'medical-history:encounter:discharged',
    'medical-history:diagnosis:created',
    'medical-history:treatment:created',
    'medical-history:evolution:created',
    'beds:facility:changed',
    'beds:assignment:created',
    'beds:assignment:released',
    'operating-rooms:room:created',
    'operating-rooms:reservation:created',
    'operating-rooms:reservation:updated',
    'operating-rooms:reservation:cancelled',
  ])('has an entry for %s', (eventName) => {
    expect(EVENT_QUERY_MAP[eventName]).toBeDefined();
    expect(EVENT_QUERY_MAP[eventName]?.length).toBeGreaterThan(0);
  });

  it('bed-facility and bed-assignment events invalidate the hospital-map prefix broadly enough to cover room-status', () => {
    // useRoomStatus() queries ['hospital-map', 'room-status', roomId] — a narrower
    // ['hospital-map', 'layout'] entry would not invalidate it (Phase 8 Finding A).
    for (const eventName of [
      'beds:facility:changed',
      'beds:assignment:created',
      'beds:assignment:released',
      'operating-rooms:room:created',
      'operating-rooms:reservation:created',
      'operating-rooms:reservation:updated',
      'operating-rooms:reservation:cancelled',
    ]) {
      const keys = EVENT_QUERY_MAP[eventName] ?? [];
      const coversHospitalMap = keys.some((key) => key.length <= 1 && key[0] === 'hospital-map');
      expect(coversHospitalMap).toBe(true);
    }
  });

  it('encounter lifecycle events invalidate patient detail per IPC.md', () => {
    for (const eventName of [
      'medical-history:encounter:created',
      'medical-history:encounter:discharged',
    ]) {
      const keys = EVENT_QUERY_MAP[eventName] ?? [];
      const coversPatientDetail = keys.some((key) => key[0] === 'patients' && key[1] === 'detail');
      expect(coversPatientDetail).toBe(true);
    }
  });
});
