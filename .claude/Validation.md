# Validation.md

## Purpose

Defines the input validation strategy across the client/server boundary — what is validated, where, and how duplication between TypeScript and Rust is kept intentional and tracked rather than accidental drift.

## Dependencies

[Rules](Rules.md) Section 11, [Security](Security.md) Section 6.

## Documents that must be read before using it

[Rules](Rules.md), [Architecture](Architecture.md), [Security](Security.md).

## Referenced By

[DefinitionOfDone](DefinitionOfDone.md), [ErrorHandling](ErrorHandling.md), [FolderStructure](FolderStructure.md), [IPC](IPC.md), [Plan](Plan.md), [README_Project](README_Project.md), [Rules](Rules.md), [Security](Security.md), [Tasks](Tasks.md), [UI](UI.md).

## Documents that must be updated if changes occur

[ErrorHandling](ErrorHandling.md) (validation error mapping), [IPC](IPC.md) (command input contracts), [Testing](Testing.md).

## Priority

6.

---

## Contents

- [1. Two-Layer Validation Model](#1-two-layer-validation-model)
- [2. Schema-as-Source-of-Truth (client side)](#2-schema-as-source-of-truth-client-side)
- [3. Intentional Client/Server Duplication](#3-intentional-clientserver-duplication)
- [4. Business-Rule Validation vs. Format Validation](#4-business-rule-validation-vs-format-validation)
- [5. Password Policy (referenced by Security.md)](#5-password-policy-referenced-by-securitymd)
- [6. Sanitization](#6-sanitization)

---

## 1. Two-Layer Validation Model

| Layer | Technology | Purpose | Trust Level |
|---|---|---|---|
| Client (TypeScript) | Zod schema, wired via `react-hook-form` resolver | Immediate UX feedback, prevents obviously-invalid submissions | UX only — **never trusted for security** |
| Server (Rust) | Manual + `validator` crate derive macros in `validation/` | Authoritative gate before any service/repository logic runs | Authoritative |

Every Tauri command's input struct has a corresponding Rust validation function executed at the top of the command handler, before the service layer is invoked (Rule 8.2). This is the enforcement point referenced by [Rules](Rules.md) 11.1.

## 2. Schema-as-Source-of-Truth (client side)

- Each module's `types/` folder defines one Zod schema per entity/command-input (e.g., `patient-schemas.ts` exports `createPatientSchema`). The TypeScript type is derived via `z.infer<typeof createPatientSchema>` — never hand-written separately (eliminates drift between type and schema, Rule 17.4).
- Forms use `zodResolver(createPatientSchema)` — the same schema instance used for both the form's live validation and the final submit-time check. No parallel/duplicate form-validation logic.

## 3. Intentional Client/Server Duplication

Because the client (Zod) and server (Rust `validator`) are different languages, the *shape* of validation rules (e.g., "MRN is required, max 50 chars," "date of birth must be in the past") is necessarily expressed twice. This is the one exception explicitly carved out by Rule 17.4, and it is tracked here:

| Entity | Rule | Zod location | Rust location |
|---|---|---|---|
| Patient | `full_name` required, 1–200 chars | `modules/patients/types/patient-schemas.ts` | `validation/patient_validation.rs` |
| Patient | `date_of_birth` must not be in the future | same | same |
| Patient | `medical_record_number` required, unique (DB-enforced) | same | same (format only; uniqueness is a DB constraint, checked at the repository/service level, not the validator) |
| User (auth) | `username` 3–50 chars, alphanumeric+`._-` | `modules/auth/types/auth-schemas.ts` | `validation/auth_validation.rs` |
| User (auth) | `password` min 12 chars, at least 1 upper/lower/digit/symbol | same | same |
| Bed assignment | `bed_id`, `patient_id`, `encounter_id` required, positive integers | `modules/beds/types/bed-schemas.ts` | `validation/bed_validation.rs` |
| OR reservation | `scheduled_start` < `scheduled_end` | `modules/operating-rooms/types/or-schemas.ts` | `validation/operating_room_validation.rs` |
| Inventory item | `quantity` >= 0, `reorder_threshold` >= 0 | `modules/inventory/types/inventory-schemas.ts` | `validation/inventory_validation.rs` |
| Inventory category | `name` required, 1–100 chars; `kind` in `medicine`/`supply`/`equipment` | same | same |
| Floor | `name` required 1–100 chars; `level_order` integer | `modules/beds/types/facility-schemas.ts` | `validation/bed_validation.rs` |
| Room | `name` required; `room_type` in the [Database](Database.md) 3.3 CHECK set; `map_x`/`map_y` floats within `0.0..=1.0` | same | same |
| Bed | `label` required 1–50 chars; `room_id` positive integer | same | same |
| Bed status change | `status` in `available`/`maintenance` only — `occupied` is rejected, since occupancy derives from `bed_assignments` ([IPC](IPC.md) Section 2) | same | same |
| Operating room | `room_id` positive integer and must reference a room whose `room_type = 'operating_room'` (referential/business check in `operating_room_service.rs`, not the format validator — see Section 4) | `modules/operating-rooms/types/or-schemas.ts` | `validation/operating_room_validation.rs` |

New entities/commands MUST add a row to this table in the same PR that adds their schema (documentation-sync rule, Rule 16.3).

## 4. Business-Rule Validation vs. Format Validation

- **Format validation** (required fields, length, type, regex) lives in the schemas/validators listed above.
- **Business-rule validation** (e.g., "a bed must be `available` to be assigned," "an encounter must be `open` to accept a new diagnosis," "an OR reservation must not overlap an existing one") lives exclusively in the Service layer (Rust), never duplicated client-side beyond disabling an obviously-invalid UI action for UX purposes (e.g., graying out an occupied bed in the picker) — the server check is always authoritative and the client-side disabling is a UX nicety, not a security control.

## 5. Password Policy (referenced by Security.md)

Minimum 12 characters, at least one uppercase, one lowercase, one digit, one symbol. Enforced identically in Zod (`auth-schemas.ts`) and Rust (`auth_validation.rs`) per Section 3's table.

## 6. Sanitization

- Text fields (names, notes, descriptions) are stored as-is (SQLite parameterized queries prevent injection, Rule 9.1) but are HTML-escaped at render time by React's default JSX text-node escaping — no `dangerouslySetInnerHTML` is used anywhere in the codebase (verified via ESLint `react/no-danger`).
- No field accepts or renders arbitrary HTML/Markdown in this MVP.

---

<!-- nav-footer -->
[← Documentation Index](README_Project.md) · [↑ Back to top](#validationmd)
