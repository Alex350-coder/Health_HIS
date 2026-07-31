# DefinitionOfDone.md

## Purpose

The single gate that determines whether a task or a phase can be marked complete. Nothing is "done" until every item below passes.

## Dependencies

[Rules](Rules.md), [Testing](Testing.md), [Security](Security.md), [Database](Database.md), [Audit](Audit.md).

## Documents that must be read before using it

[Rules](Rules.md), [Testing](Testing.md).

## Referenced By

[DevelopmentWorkflow](DevelopmentWorkflow.md), [Plan](Plan.md), [Progress](Progress.md), [README_Project](README_Project.md), [Rules](Rules.md), [Tasks](Tasks.md), [Testing](Testing.md), [UI](UI.md).

## Documents that must be updated if changes occur

[Plan](Plan.md) (Phase Exit Criteria reference this document), [Tasks](Tasks.md) (Completion Criteria reference this document).

## Priority

8.

---

## 1. Task-Level Definition of Done

A single task (as defined in [Tasks](Tasks.md)) is done only when ALL of the following are true:

- [ ] Code implements exactly the task's stated Objective — nothing more, nothing less (no scope creep, no speculative abstraction, per [CodingStandards](CodingStandards.md) Section 4).
- [ ] **Build succeeds**: `cargo build` (Rust) and `npm run build` (Vite/TS) both complete without error. From Phase 3 onward, `npm run tauri build --debug` must also succeed — a project that lints and type-checks but does not produce a runnable binary is not done.
- [ ] `cargo fmt --check` and `cargo clippy -D warnings` pass (if Rust code changed).
- [ ] `eslint --max-warnings=0` and `tsc --noEmit` pass (if TS/React code changed).
- [ ] All new/changed code has tests per [Testing](Testing.md), and the relevant coverage threshold (Section 3 of [Testing](Testing.md)) is met.
- [ ] `cargo test` and `vitest` both pass locally and in CI.
- [ ] Every rule in [Rules](Rules.md) applicable to the changed code is satisfied — checked against the specific sections relevant to the change (e.g., a new Tauri command checks Rules 8.x and 3.5).
- [ ] If the task touches the database: migration is forward-only, follows [Database](Database.md) Section 5 naming, and the migration test (Section 1 of [Testing](Testing.md)) passes.
- [ ] If the task touches a security-relevant path (auth, session, input crossing IPC): [Security](Security.md) and [Validation](Validation.md) requirements are met, verified against the Section 3 duplication table in [Validation](Validation.md).
- [ ] If the task performs a critical action per [Audit](Audit.md) Section 2: an audit write is present and covered by a test.
- [ ] No mock data, no commented-out code, no dead code, no unused files introduced (Rule 17).
- [ ] [New_files](New_files.md) updated with every new file and its purpose, in the same commit.
- [ ] [Progress](Progress.md) updated: the task's row is set to `Done` (or `Blocked` with a reason) in the active phase's task table, in the same commit — not batched at the end of a session.
- [ ] Relevant documentation updated in the same commit if the task changed architecture, schema, security behavior, or contracts (Rule 16.3) — e.g., a new command updates [IPC](IPC.md), a new table updates [Database](Database.md).

## 2. Phase-Level Definition of Done

A phase (as defined in [Plan](Plan.md)) is done only when:

- [ ] Every task within the phase meets Section 1 above.
- [ ] Full CI pipeline (Section 7 of [Testing](Testing.md)) passes on the phase's final state, including `npm audit` / `cargo audit` with no unaddressed high/critical findings.
- [ ] No mock-data detection findings (manual grep/review for hardcoded arrays, `faker` usage in non-dev-seed paths).
- [ ] No dead-code/unused-file findings (`ts-prune`/`knip`, `clippy` dead_code lint).
- [ ] The phase's own Exit Criteria and Regression Checklist in [Plan](Plan.md) pass in full.
- [ ] [Progress](Progress.md) updated to reflect the phase's completed status.
- [ ] A manual review pass confirms the phase's UI (if any) satisfies the loading/empty/error triad ([UI](UI.md) Section 6) and the accessibility checklist ([UI](UI.md) Section 8).
- [ ] If the phase introduced or modified any workflow step from the patient-centered workflow ([README_Project](README_Project.md) Section 2), an end-to-end walkthrough of that step (manual or via the E2E suite, [Testing](Testing.md) Section 4) has been performed and passes.

## 3. Project-Level Definition of Done (MVP Complete)

- [ ] Every phase in [Plan](Plan.md) meets Section 2 above.
- [ ] The full patient-centered workflow (registration through permanent history preservation) works end-to-end via the `patient-admission-flow.spec.ts` E2E test ([Testing](Testing.md) Section 4).
- [ ] Every module listed in [README_Project](README_Project.md) Section 3 is implemented and reachable per [Routes](Routes.md).
- [ ] `audit_verify_integrity` passes against the accumulated audit log from all E2E runs.
- [ ] No document in `.claude/` contradicts the current codebase (spot-checked during final review — code and docs must not have diverged, Rule 16.3).

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#definitionofdonemd)
