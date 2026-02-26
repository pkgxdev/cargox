# AGENTS: cargox

Public repository for Cargo ecosystem command execution tooling.

## Core Commands

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

## Always Do

- Keep install and version resolution behavior stable.
- Preserve clear error messaging for install fallback paths.

## Ask First

- Changes to release script semantics (`scripts/release.py`).
- Any default behavior change for install backend selection.

## Never Do

- Never hide install failures.
- Never remove safety checks around command invocation.
