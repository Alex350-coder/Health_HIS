import { existsSync, rmSync } from 'node:fs';
import { join } from 'node:path';

const APP_IDENTIFIER = 'com.healthproject.his';
const DB_FILE_NAME = 'health.db';

/**
 * Deletes the app's SQLCipher database file so `bootstrap.spec.ts`-style scenarios start from
 * a state with no admin user (Testing.md Section 4, scenario 0; Rules.md 17.1 forbids seeding
 * this state any other way). Windows-only: mirrors Tauri's `app_data_dir()` resolution, which
 * uses `%APPDATA%` (roaming) on this platform.
 */
export function resetDatabase(): void {
  const appData = process.env.APPDATA;
  if (!appData) {
    throw new Error('APPDATA environment variable is not set; cannot locate the app database.');
  }
  const dbPath = join(appData, APP_IDENTIFIER, DB_FILE_NAME);
  if (existsSync(dbPath)) {
    rmSync(dbPath);
  }
}
