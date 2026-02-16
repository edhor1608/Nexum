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

## Additional Work (Milestones 4-5)
- Added niri-native shell planning module driven by restore plans.
- Added attention routing policy module aligned to urgency classes.
- Added control-plane execution plan composer combining routing, shell, and attention.
- Added shadow parity comparison module for internal cutover validation.
- Added runtime event store (SQLite) for structured observability events.
- Extended `nexumctl` with `flags` and `parity` commands.

## New Test Coverage (Milestones 4-5)
- Unit tests: attention routing and parity scoring behavior.
- Integration tests: control-plane execution composition and event persistence.
- Snapshot tests: control-plane execution contract and parity report contract.
- E2E tests: `nexumctl flags set/show` and `nexumctl parity compare` workflows.

## Additional Work (Milestone 6)
- Added explicit niri adapter boundary (`NiriAdapter`) to decouple plan generation from execution backend.
- Added `execute_shell_plan` with fail-fast semantics.
- Added `render_shell_script` for deterministic command rendering and review.
- Extended `nexumctl` with `shell render` command for quick operator inspection.

## New Test Coverage (Milestone 6)
- Adapter unit tests for command ordering and failure stop behavior.
- Snapshot contract for rendered niri shell script.
- CLI e2e test for shell script rendering.
