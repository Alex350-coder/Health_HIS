# Hospital Information System

A secure, offline-first desktop Hospital Information System (HIS) MVP.

**Tauri 2.x** · **React 18 + TypeScript** · **Rust** · **SQLite + SQLCipher**

> This repository is planned and governed by the documentation framework in
> [`.claude/`](.claude/README_Project.md). Start at [CLAUDE.md](CLAUDE.md) for orientation and
> [`.claude/Rules.md`](.claude/Rules.md) for the binding rules.

## Prerequisites

- Node.js LTS (20 or newer)
- Rust stable toolchain (`rustup`)
- Platform Tauri prerequisites — WebView2 on Windows, `libwebkit2gtk-4.1-dev` and friends on
  Linux. See the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/).

## Getting started

```bash
npm install
npm run tauri dev
```

## Commands

| Command                                          | Purpose                                            |
| ------------------------------------------------ | -------------------------------------------------- |
| `npm run tauri dev`                              | Run the full app in dev mode.                      |
| `npm run build`                                  | Type-check and build the frontend bundle.          |
| `npm run tauri build`                            | Produce the platform-native installer.             |
| `npm run lint`                                   | ESLint (`--max-warnings=0`).                       |
| `npm run typecheck`                              | `tsc --noEmit`.                                    |
| `npm run test`                                   | Vitest.                                            |
| `npm run test:coverage`                          | Vitest with coverage thresholds.                   |
| `cargo fmt --check` / `cargo clippy -D warnings` | Rust formatting and lints (in `src-tauri/`).       |
| `cargo test`                                     | Rust unit and integration tests (in `src-tauri/`). |

## Quality gates

CI runs the gates in the fixed order defined by
[`.claude/Testing.md`](.claude/Testing.md) Section 7. A pre-commit hook (`husky` +
`lint-staged`) runs Prettier and ESLint on staged TypeScript, plus `cargo fmt`/`clippy` when
Rust files are staged. Hooks are never bypassed with `--no-verify`.

## License

Not licensed for distribution — an educational MVP.
