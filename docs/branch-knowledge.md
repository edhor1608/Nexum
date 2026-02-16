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

## Additional Work (Milestone 7)
- Added self-signed TLS certificate lifecycle manager (`tls` module).
- Implemented cert generation, reuse, metadata persistence, and threshold-based rotation.
- Extended `nexumctl` with `tls ensure` and `tls rotate` commands.

## New Test Coverage (Milestone 7)
- TLS lifecycle behavior tests (create/reuse/rotate/no-rotate).
- TLS record snapshot contract.
- TLS CLI e2e validation.

## Additional Work (Milestone 8)
- Added cutover gate evaluator for routing/restore/attention capability rollout.
- Added gate application flow that updates flags only when decision allows.
- Extended `nexumctl` with `cutover apply` command.

## New Test Coverage (Milestone 8)
- Cutover gate unit tests for allow/deny scenarios.
- Cutover integration test with parity + flags application.
- Cutover decision snapshot contract.
- Cutover CLI e2e flow validating flag file mutation.

## Additional Work (Milestone 9)
- Added restore-runner orchestration module (`runflow`) for an executable control-plane restore slice.
- Flow now composes restore planning, route registration, TLS ensure, shell script rendering, and event persistence.
- Extended `nexumctl` with `run restore` command.

## New Test Coverage (Milestone 9)
- Restore runner integration test for script output + event persistence.
- Restore runner snapshot contract.
- Restore runner CLI e2e flow.

## Additional Work (Milestone 10)
- Extended `nexumctl` with daemon-backed routing commands: `health`, `register`, `resolve`, `remove`, and `list`.
- Added CLI runtime bridge that executes async routing socket calls from synchronous command handlers.
- Added routing usage/help entries to keep operator surface explicit.

## New Test Coverage (Milestone 10)
- Routing CLI e2e test validating daemon interaction across health/register/resolve/remove lifecycle.
- Routing CLI snapshot contract for JSON outcome schema (`RouteOutcome::Registered`).

## Additional Work (Milestone 11)
- Extended restore runner to support daemon-backed routing registration via optional routing socket.
- Added `--routing-socket` support to `nexumctl run restore`.
- Kept fallback behavior for local/in-process routing path when no socket is provided.

## New Test Coverage (Milestone 11)
- Restore runner daemon integration test validating route registration through `nexumd`.
- Restore runner daemon conflict test validating domain-claim rejection path.
- Restore runner CLI e2e test validating `run restore` + `routing resolve` end-to-end with daemon socket.

## Additional Work (Milestone 12)
- Added explicit acceptance-level e2e coverage for parallel restore reliability and restore latency budget.
- Validated daemon-backed routing list behavior after concurrent restore operations.

## New Test Coverage (Milestone 12)
- Parallel e2e scenario: 5 concurrent `run restore` operations register 5 deterministic routes.
- Budget e2e scenario: single `run restore` completes under 10 seconds wall-clock.

## Additional Work (Milestone 13)
- Added identity policy module for browser launch strategy with collision-aware profile fallback.
- Extended restore runner input/CLI with `--identity-collision` to activate profile fallback path.
- Wired restore shell script generation to swap default browser launch command when collision is signaled.

## New Test Coverage (Milestone 13)
- Identity policy unit tests for domain-default and collision fallback command generation.
- Restore runner CLI e2e for collision-driven profile fallback behavior.

## Additional Work (Milestone 14)
- Added isolation policy module implementing mode escalation for collision, high-risk secret workflow, or explicit override.
- Extended restore flow summary with `run_mode` for operator-visible isolation state.
- Extended `nexumctl run restore` with `--high-risk-secret` and `--force-isolated` toggles.

## New Test Coverage (Milestone 14)
- Isolation policy unit matrix for default mode and all escalation triggers.
- Restore CLI e2e for high-risk secret mode escalation.
- Restore runner snapshot contract now locks `run_mode` output.

## Additional Work (Milestone 15)
- Added runtime metadata module for capsule-scoped env export set and terminal process label conventions.
- Restore runner now prepends capsule metadata exports to generated shell script before execution commands.

## New Test Coverage (Milestone 15)
- Runtime metadata unit tests for env contract and process label format.
- Restore CLI e2e validates shell script includes `NEXUM_*` exports and process label.
- Restore runner snapshot contract updated with metadata export prelude.

## Additional Work (Milestone 16)
- Added capsule lifecycle state model (`creating`, `ready`, `restoring`, `degraded`, `archived`) to core capsule contract.
- Extended capsule persistence with state column (including migration path for existing DBs).
- Added capsule lifecycle CLI operations: `capsule rename` and `capsule set-state`.

## New Test Coverage (Milestone 16)
- Store integration test for explicit lifecycle state transition persistence.
- Capsule lifecycle CLI e2e for rename + state transition flow.
- Capsule/store snapshots updated to lock state field in contract exports.
