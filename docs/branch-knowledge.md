# Branch Knowledge - P19 Initial Implementation

## Problem We Solved
- Bootstrapped Nexum as runnable control-plane code with strict TDD.
- Implemented core capsule identity/restore/resource behavior.
- Implemented custom routing daemon over Unix socket JSON API.
- Implemented persistent capsule state store and local migration flags.

## What We Tried
- Began with behavior tests first for each milestone (red/green).
- Added snapshot tests to lock API/data contracts.
- Added integration and e2e tests to validate real process/socket flows.

## What Worked
- Rust single-package workspace with two binaries (`nexumd`, `nexumctl`).
- Tokio Unix socket daemon model with in-memory routing state.
- SQLite-backed store for deterministic capsule persistence.
- Local TOML flags file for shadow/cutover control.

## What Did Not Work / Adjustments
- `cargo insta` CLI not installed; snapshots accepted by promoting `*.snap.new`.
- Runtime env lookup for `nexumctl` binary in e2e was unreliable; switched to `assert_cmd` cargo binary resolution.

## Key Design Decisions in Code
- Domain identity format: `<slug>.nexum.local`.
- TLS mode in routing entries: `self_signed` marker for v1 policy.
- Capsule slug is immutable after creation (rename affects display name only).
- Restore flow has deterministic step order and 9.5s target budget.
- Cutover flags default to shadow mode enabled, control-plane cutovers off.

## Test Coverage Added
- Unit tests: slug normalization, route conflict handling, flag defaults.
- Property tests: slug DNS-safety invariants.
- Integration tests: socket API and SQLite persistence behavior.
- Snapshot tests: capsule/restore/routing/store output contracts.
- E2E tests: `nexumd` socket interaction and `nexumctl` create/list workflow.

## Milestone Commits
- `8df1b39` - core capsule/restore/resource contracts
- `17543b7` - routing daemon + socket API
- `147de64` - store/flags + `nexumctl`
