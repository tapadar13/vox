# Contributing to Vox

Thank you for helping improve Vox. Keep changes small, local-first, and testable.

## Setup

Install Node.js 22+, Rust stable, and Xcode Command Line Tools, then run:

```bash
npm ci
npm run tauri dev
```

The UI can run with `npm run dev`, but native commands require the Tauri process.
Never add a model weight, updater private key, transcript database, or personal log
file to the repository.

## Design rules

- Put product decisions and state transitions in the pure Rust domain.
- Keep platform/vendor details inside adapters behind existing ports.
- Treat backend `AppState` as the frontend's source of truth.
- Keep the CPAL callback bounded; do allocation and inference on workers.
- Preserve the no-audio-on-disk and no-cloud-inference guarantees.
- Add a migration rather than editing an applied SQLite schema in place.
- Validate downloads before atomically moving them into their final path.

## Before opening a pull request

```bash
npm test
npm run build
cd src-tauri
cargo fmt --check
cargo test --no-default-features
cargo clippy --no-default-features --all-targets -- -D warnings
cargo check
cargo check --no-default-features --features parakeet
```

Tests should concentrate on observable behavior and pure logic. Avoid mocks around
vendor APIs unless they protect a meaningful integration boundary.

## Commit and pull-request scope

Prefer focused commits that each explain one coherent change. A pull request should
describe user-visible behavior, validation performed, privacy/network changes, and
any macOS permission implications. Include screenshots only when the UI changed.

Report security-sensitive findings through the private process in `SECURITY.md`.
