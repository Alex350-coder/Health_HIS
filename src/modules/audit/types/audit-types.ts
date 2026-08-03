/** Mirrors `src-tauri/src/models/audit.rs::AuditLogEntry` (Database.md Section 3.7 / Audit.md). */
export interface AuditLogEntry {
  id: number;
  timestamp: string;
  userId: number | null;
  action: string;
  entityType: string;
  entityId: number | null;
  beforeState: string | null;
  afterState: string | null;
  result: string;
  prevHash: string;
  rowHash: string;
}

export interface AuditListFilter {
  entityType?: string;
  userId?: number;
  from?: string;
  to?: string;
  limit: number;
  offset: number;
}

export interface ChainVerification {
  isValid: boolean;
  brokenAtId: number | null;
}
