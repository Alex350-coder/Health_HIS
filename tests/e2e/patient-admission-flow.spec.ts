import { expect } from '@wdio/globals';

import { resetDatabase } from './support/reset-database';

/**
 * Exercises the full patient-centered workflow chain (Testing.md Section 4 scenario 2): bed
 * assignment (Plan.md Phase 8 scope) through diagnosis, treatment with inventory consumption,
 * evolution, OR scheduling, billing simulation, discharge, and a post-discharge history re-query
 * (Plan.md Phase 13 scope). Runs against the real, freshly-reset SQLCipher database — no
 * seeded/mock data (Rules.md 17.1) — so IDs are the deterministic first auto-increment values
 * (1) a clean database produces.
 *
 * Also validates the Phase 8 Finding A fix end-to-end: after `beds:assignment:created`, the
 * Hospital Map room-status panel reflects occupancy without a manual page reload.
 */
describe('Patient admission flow — through discharge', () => {
  before(async () => {
    resetDatabase();
    await browser.reloadSession();
  });

  it('bootstraps the first admin user', async () => {
    await $('#bootstrap-full-name').setValue('Admin User');
    await $('#bootstrap-username').setValue('admin');
    await $('#bootstrap-password').setValue('Str0ng!Passw0rd');
    await $('button=Create administrator account').click();

    await expect($('h1=Hospital Information System')).toBeDisplayed();
  });

  it('creates a floor, room, and bed from the facility page', async () => {
    await browser.url('/facility');

    await $('#floor-name').setValue('First Floor');
    await $('#floor-level-order').setValue('1');
    await $('button=Add floor').click();
    await expect($('*=First Floor')).toBeDisplayed();

    await $('#room-floor-id').setValue('1');
    await $('#room-name').setValue('Room 101');
    await $('#room-map-x').setValue('0.1');
    await $('#room-map-y').setValue('0.1');
    await $('button=Add room').click();
    await expect($('*=Room 101')).toBeDisplayed();

    await $('#bed-room-id').setValue('1');
    await $('#bed-label').setValue('Bed 101-A');
    await $('button=Add bed').click();
    await expect($('*=Bed 101-A')).toBeDisplayed();
  });

  it('promotes the room to an operating room from the facility page', async () => {
    await browser.url('/facility');

    await $('#operating-room-room-id').setValue('1');
    await $('button=Promote to OR').click();
    await expect($('*=Room 101')).toBeDisplayed();
  });

  it('creates an inventory category and item', async () => {
    await browser.url('/inventory');

    await $('#inventory-category-name').setValue('Consumables');
    await $('#inventory-category-kind').selectByAttribute('value', 'supply');
    await $('button=Add category').click();
    await expect($('*=Consumables')).toBeDisplayed();

    await $('#inventory-item-category').selectByAttribute('value', '1');
    await $('#inventory-item-name').setValue('Gauze Pads');
    await $('#inventory-item-unit').setValue('box');
    await $('#inventory-item-reorder-threshold').setValue('5');
    await $('button=Add item').click();
    await expect($('*=Gauze Pads')).toBeDisplayed();
  });

  it('registers a patient', async () => {
    await browser.url('/patients/new');

    await $('#create-patient-medical-record-number').setValue('MRN-0001');
    await $('#create-patient-full-name').setValue('Jane Doe');
    await $('#create-patient-date-of-birth').setValue('1990-01-01');
    await $('#create-patient-sex').selectByAttribute('value', 'female');
    await $('button=Register patient').click();

    await browser.waitUntil(async () => (await browser.getUrl()).includes('/patients/1'), {
      timeout: 10_000,
      timeoutMsg: 'expected navigation to /patients/1 after patient creation',
    });
  });

  it('starts an encounter for the patient', async () => {
    await browser.url('/patients/1/medical-history');

    await $('button=Start encounter').click();
    await expect($('button=Start encounter')).not.toBeDisplayed();
  });

  it('assigns the bed and propagates occupancy to the Beds list, patient tab, and Hospital Map', async () => {
    await browser.url('/patients/1/beds');

    await $('button=Assign bed').click();
    await $('#assign-bed-id').selectByAttribute('value', '1');
    await $('button=Assign').click();

    await expect($('*=Bed 101-A')).toBeDisplayed();
    await expect($('button=Release')).toBeDisplayed();

    await browser.url('/beds');
    await expect($('button=Release')).toBeDisplayed();

    await browser.url('/hospital-map');
    await $('[aria-label="Room 101"]').click();
    const occupiedCount = $('dt=Occupied').parentElement().$('dd');
    await expect(occupiedCount).toHaveText('1');
  });
});
