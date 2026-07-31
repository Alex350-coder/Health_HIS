# Security.md

## Purpose

Defines the complete security posture: authentication, session management, secrets, encryption, rate limiting, and defense-in-depth measures.

## Dependencies

[Rules](Rules.md) Section 10, [Architecture](Architecture.md) Section 1, [Database](Database.md) Section 1–2.

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md), [Database](Database.md).

## Referenced By

[Architecture](Architecture.md), [Audit](Audit.md), [Database](Database.md), [DefinitionOfDone](DefinitionOfDone.md), [DevelopmentWorkflow](DevelopmentWorkflow.md), [ErrorHandling](ErrorHandling.md), [FolderStructure](FolderStructure.md), [Glossary](Glossary.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Routes](Routes.md), [Rules](Rules.md), [StateManagement](StateManagement.md), [Tasks](Tasks.md), [Testing](Testing.md), [Validation](Validation.md).

## Documents that must be updated if changes occur

[Audit](Audit.md) (security events are audited), [ErrorHandling](ErrorHandling.md) (auth/authz error categories), [Validation](Validation.md), [Testing](Testing.md) (security test suite).

## Priority

5.

---

## Contents

- [1. Defense in Depth — Layered Model](#1-defense-in-depth--layered-model)
- [2. Password Hashing](#2-password-hashing)
- [3. Rate Limiting & Brute-Force Protection](#3-rate-limiting--brute-force-protection)
- [4. Session Management](#4-session-management)
- [5. Secrets Management](#5-secrets-management)
- [6. Input Validation (cross-reference)](#6-input-validation-cross-reference)
- [7. Least Privilege](#7-least-privilege)
- [8. Audit Integrity](#8-audit-integrity)
- [9. Secure Defaults](#9-secure-defaults)
- [10. Security Testing](#10-security-testing)

---

## 1. Defense in Depth — Layered Model

1. **Process boundary**: WebView (untrusted, renders remote-content-like surface) cannot reach the filesystem or DB. Only the Rust core can. See [Architecture](Architecture.md) Section 1.
2. **Capability allowlisting**: Tauri `capabilities/*.json` grants each window only the specific commands/plugins it needs (Rule 8.1). No wildcard `"*"` permissions.
3. **Command-level input validation**: every command re-validates input server-side regardless of client-side Zod validation (Rule 8.2, Rule 11.1).
4. **Parameterized queries only** (Rule 9.1) — eliminates SQL injection as an attack class entirely.
5. **Encryption at rest**: SQLCipher AES-256 for the whole DB file (Rule 10.4, [Database](Database.md) Section 1).
6. **Session validation on every command**: each command (except `auth_login`, `auth_logout`) requires a valid, non-expired session token passed via Tauri managed state, checked against `sessions` before the service layer runs.
7. **Audit trail**: every security-relevant and clinically-relevant action is recorded immutably. See [Audit](Audit.md).

## 2. Password Hashing

- Algorithm: **Argon2id** (`argon2` crate), never Argon2i/Argon2d alone (Argon2id is fixed as the resistant hybrid, Rule 10.1).
- Parameters (tuned for desktop, single active user, must resist offline attack if DB file is exfiltrated): memory cost `19456` KiB (19 MiB), iterations `2`, parallelism `1` — OWASP-recommended baseline for Argon2id; documented here so any future tuning is a deliberate, reviewed change, not an ad hoc edit in code.
- Salt: unique per password, generated via the crate's CSPRNG, stored as part of the encoded Argon2id hash string (standard PHC format) in `users.password_hash`.
- Plaintext passwords: never logged (Rule 12.1), never stored, never sent back in any command response.

## 3. Rate Limiting & Brute-Force Protection

- Enforced in `security/rate_limit.rs`, invoked by `auth_service.rs` before every login attempt.
- Per-account counter: `users.failed_login_attempts`. On failure, increment; on success, reset to 0.
- Lockout: after 5 consecutive failures, `users.locked_until` is set to `now + lockout_duration`. Lockout duration escalates: 1st lockout = 1 minute, 2nd = 5 minutes, 3rd+ = 15 minutes (exponential backoff, capped).
- While locked, `auth_login` returns `AppError::AccountLocked` (see [ErrorHandling](ErrorHandling.md)) without touching the password hash comparison (avoids unnecessary work and timing signal beyond "locked").
- Every attempt (success/failure/locked) is written to `audit_log` (action `auth.login`), per [Audit](Audit.md).
- No global/IP-based rate limiting needed (single-device desktop app, not network-exposed).

## 4. Session Management

- On successful login, `auth_service.rs` generates a 256-bit random token (`rand::rngs::OsRng`), returns the raw token to the frontend once, and stores only `SHA-256(token)` in `sessions.token_hash`.
- The frontend holds the raw token in memory only (Zustand auth slice) — never written to `localStorage`/disk. On app restart, the user must re-authenticate (no "remember me" persistence in the MVP; a future phase could add OS-keychain-backed persistence, tracked as a backlog item, not built now).
- Session expiry: absolute lifetime 8 hours; idle timeout 15 minutes (checked against `last_active_at`, updated on each authenticated command).
- Every authenticated command handler validates the session token hash against `sessions`, checks `expires_at` and idle timeout, and rejects with `AppError::Unauthorized` if invalid/expired — this check lives in one shared middleware-style helper (`security/session.rs::require_session`), called by every command (Rule: no duplicated logic).
- Logout deletes the session row immediately (not just client-side token discard).

## 5. Secrets Management

- The SQLCipher database encryption key is a 256-bit value generated on first run and stored in the OS-native credential store via the `keyring` crate (Windows Credential Manager / macOS Keychain / Linux Secret Service).
- No secret, key, or credential is ever committed to source control, written to a `.env` file that ships with the app, or hardcoded. CI runs a secret-scanning step (e.g., `gitleaks`) on every PR (Rule 10.5).
- If the OS keychain is unavailable (e.g., headless CI test environment), tests use an explicit, clearly-named test-only in-memory key path — never the production key derivation code path.

## 6. Input Validation (cross-reference)

Full strategy in [Validation](Validation.md). Security-relevant summary: server-side (Rust) validation is authoritative; client-side (Zod) validation is a UX convenience only and must never be trusted alone (Rule 11.1).

## 7. Least Privilege

- Tauri capability files grant the minimum command set per window/context (Rule 8.1).
- The `users.role` column (see [Database](Database.md) 3.1) is used for audit attribution and future extensibility, not for UI-level access branching — every authenticated user sees the same application, per the project spec, minimizing the access-control surface to a single gate: authenticated vs. not.
- The OS process itself should be run without elevated/admin privileges; the Tauri bundle configuration does not request elevation.

## 8. Audit Integrity

Full spec in [Audit](Audit.md). Security-relevant summary: `audit_log` is hash-chained (`prev_hash`/`row_hash`) so any retroactive tampering (including via direct file access to the SQLite file, bypassing the app) is detectable; a SQL trigger denies `UPDATE`/`DELETE` on the table as an additional layer.

## 9. Secure Defaults

- New user accounts are created only by an existing authenticated user through `auth_service::create_user` (no public self-registration screen — appropriate for an internal hospital system) and default to `is_active = 1` but must have a strong password meeting the policy in [Validation](Validation.md). The single exception is first-run bootstrap, specified in Section 9.1 below.
- All Tauri window configurations disable `devtools` in release builds and disable arbitrary navigation (`tauri.conf.json` CSP configured to only allow the app's own asset origin).
- Content Security Policy (CSP) is explicitly configured in `tauri.conf.json` (no `unsafe-inline`/`unsafe-eval` for scripts) to reduce XSS blast radius even though the WebView cannot reach the DB directly (defense in depth per Section 1).

### 9.1 First-Run Bootstrap (the only unauthenticated write path)

> [!CAUTION]
> This is the **only** write path in the entire system reachable without a session. Any change to this section is a change to the system's authentication boundary and requires re-reviewing Sections 1–8 in the same PR.

**The problem.** Section 9 permits user creation only by an authenticated user, and [Rules](Rules.md) 17.1 forbids seeding a hardcoded default account. Taken together, an empty `users` table is a permanent deadlock: nobody can log in, so nobody can ever be created. The system needs exactly one, tightly-bounded way out.

**The mechanism.** Two unauthenticated commands ([IPC](IPC.md) Section 2):

- `auth_bootstrap_status` — returns `{ needsBootstrap: bool }`, true iff `SELECT COUNT(*) FROM users = 0`. It reveals one bit: whether this installation has been initialised. On an initialised system it returns `false` and nothing else, so it is not a user-enumeration or reconnaissance surface.
- `auth_bootstrap_admin` — creates the first user, with `role = 'admin'`, and issues a session.

**The gate.** `auth_bootstrap_admin` re-checks `COUNT(*) FROM users = 0` **inside the same transaction as the `INSERT`**, not before it. Checking outside the transaction would leave a TOCTOU window in which two concurrent calls could each observe an empty table and each create an "first" admin. The check and the insert must be atomic; the `users.username UNIQUE` constraint is a backstop, not the primary control. Once any user exists, this command returns `AppError::Conflict` forever — it does not become a second registration path, and it can never be used to add an account to a running hospital.

**Why this does not weaken the posture.** The bootstrap window is open only on a database with zero users, which is only true before the system holds any PHI at all. An attacker who reaches an un-bootstrapped installation has local access to a hospital workstation and an empty encrypted database containing no patient data — strictly less than they would gain from simply reading the file. The window closes permanently at first use and cannot be reopened without deleting the database.

**Additional requirements.**
- The bootstrap password is subject to the full password policy ([Validation](Validation.md) Section 5) — no relaxation because it is "just setup."
- Both commands are rate-limited by the same mechanism as `auth_login` (Section 3), keyed by installation rather than account, to prevent bootstrap-attempt flooding.
- `auth_bootstrap_admin` writes an audit row (`user.create`, with `user_id` referencing the account it just created, since no prior actor exists) — the audit chain's genesis entry ([Audit](Audit.md) Section 4).
- The `/setup` route ([Routes](Routes.md) Section 3) is reachable **only** when `auth_bootstrap_status` returns true, and redirects to `/login` otherwise.
- Integration tests required ([Testing](Testing.md) Section 1 "Security"): bootstrap succeeds on an empty DB; a second bootstrap attempt is rejected with `Conflict`; bootstrap is rejected on a DB with any existing user; the created user can immediately log in; the weak-password case is rejected.

## 10. Security Testing

See [Testing](Testing.md) Section "Security Testing" for the concrete test list (rate limit exhaustion, session expiry, SQL injection attempt via crafted input, capability boundary tests).

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#securitymd)
