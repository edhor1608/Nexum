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

## Additional Work (Milestone 17)
- Added persisted capsule port allocation table in SQLite store (`capsule_ports`).
- Added store-level deterministic `allocate_port` and `release_ports` operations with stable per-capsule allocation.
- Extended capsule CLI with `allocate-port` and `release-ports`, and exposed `allocated_ports` in `capsule list`.

## New Test Coverage (Milestone 17)
- Store integration test for stable allocation + reuse after release.
- Capsule ports CLI e2e for allocate/list/release behavior.

## Additional Work (Milestone 18)
- Implemented degraded restore path when routing transport is unavailable (socket/connect/protocol issues).
- Preserve hard failure for domain conflicts while allowing degraded continuation for transport-level route unavailability.
- Extended restore summary contract with `degraded` and `degraded_reason`.

## New Test Coverage (Milestone 18)
- Restore runner degraded integration test for unavailable routing socket behavior.
- Restore CLI e2e degraded-path test via missing routing socket.
- Restore runner snapshot updated to lock degraded fields in summary contract.

## Additional Work (Milestone 19)
- Added event-level critical count query to event store.
- Added `nexumctl cutover apply-from-events` to compute critical-event gate input directly from runtime events DB.
- Wired event-driven cutover path into existing cutover evaluator and flag application flow.

## New Test Coverage (Milestone 19)
- Cutover CLI e2e tests for event-driven allow and deny cases based on critical event threshold.

## Additional Work (Milestone 20)
- Added `repo_path` to capsule contract and SQLite persistence.
- Added migration-safe repo path column handling for existing capsule stores.
- Extended capsule CLI with `set-repo` and optional `--repo-path` on create.
- Exposed `repo_path` in `capsule list` output.

## New Test Coverage (Milestone 20)
- Store integration test for repo-path persistence.
- Capsule lifecycle CLI e2e updated to validate repo-path mutation.
- Capsule and store snapshots updated for repo-path contract field.

## Additional Work (Milestone 21)
- Added aggregated runtime events summary in event store (global totals + per-capsule rollup).
- Added `nexumctl events summary` command for supervisor-level health visibility.

## New Test Coverage (Milestone 21)
- Events CLI e2e validating aggregate totals and capsule-level critical counts.

## Additional Work (Milestone 22)
- Added restore lifecycle state transitions through optional capsule store path wiring.
- `run restore` now accepts `--capsule-db` and updates capsule state to `restoring` before execution.
- Final restore state now persists as `ready` on success and `degraded` on degraded outcomes or hard route errors.

## New Test Coverage (Milestone 22)
- Restore CLI e2e test validating successful restore transitions state from `archived` to `ready`.
- Restore CLI e2e test validating routing-unavailable restore transitions state to `degraded`.

## Additional Work (Milestone 23)
- Added `nexumctl run restore-capsule` command to execute restore from persisted capsule metadata.
- New restore path loads capsule record from SQLite and derives defaults from `repo_path`:
  - terminal: `cd <repo_path> && nix develop`
  - editor: `<repo_path>`
  - browser: `https://<slug>.nexum.local`
- Enforced explicit surface input when no `repo_path` exists.

## New Test Coverage (Milestone 23)
- Restore CLI e2e validating metadata-derived restore defaults from capsule store.
- Restore CLI e2e validating strict failure when `repo_path` is missing and no explicit terminal/editor are provided.

## Additional Work (Milestone 24)
- Added filtered event query support in runtime event store (`list_recent`).
- Added `nexumctl events list` command with optional filters:
  - `--capsule-id`
  - `--level`
  - `--limit`
- Event list output is ordered by newest first (`ts_unix_ms` then insertion id).

## New Test Coverage (Milestone 24)
- Observability integration test for filtered + limited recent event queries.
- Events CLI e2e for `events list` filter and limit behavior.

## Additional Work (Milestone 25)
- Added explicit Stead ingress boundary via `stead::DispatchEvent` envelope parser.
- Added `nexumctl stead dispatch` command to execute restore from Stead event payloads plus capsule metadata.
- Dispatch path supports event-driven isolation flags (`identity_collision`, `high_risk_secret_workflow`, `force_isolated_mode`) and optional surface overrides.

## New Test Coverage (Milestone 25)
- Stead CLI e2e for successful envelope-driven restore dispatch.
- Stead CLI e2e for strict rejection of invalid event payloads.

## Additional Work (Milestone 26)
- Added `nexumctl supervisor status` command for unified control-plane health visibility.
- Status report now combines:
  - current cutover flags
  - capsule lifecycle inventory (total/degraded/archived)
  - per-capsule critical-event counts
  - last event metadata per capsule (level/message/timestamp)

## New Test Coverage (Milestone 26)
- Supervisor CLI e2e validating aggregated capsule health + event health report contract.

## Additional Work (Milestone 27)
- Added `nexumctl supervisor blockers` command for focused triage output.
- Blocker report includes capsules matching either:
  - degraded lifecycle state
  - critical-event threshold breach
- Output now carries deterministic blocker reasons (`state_degraded`, `critical_events_threshold`).

## New Test Coverage (Milestone 27)
- Supervisor CLI e2e validating blocker selection and reason tags for degraded/critical capsules.

## Additional Work (Milestone 28)
- Added `nexumctl cutover apply-from-summary` command for global runtime gate evaluation.
- New path computes critical-event input from event-store summary (`critical_events` total), then reuses existing cutover evaluator/flag application flow.

## New Test Coverage (Milestone 28)
- Cutover events CLI e2e for global-summary allow case (within threshold).
- Cutover events CLI e2e for global-summary deny case (threshold exceeded).

## Additional Work (Milestone 29)
- Added snapshot contract coverage for supervisor control-plane surfaces.
- Locked YAML contracts for:
  - `supervisor status` payload shape
  - `supervisor blockers` payload shape

## New Test Coverage (Milestone 29)
- Snapshot tests for supervisor status/blockers output contracts.

## Additional Work (Milestone 30)
- Added `nexumctl capsule export --format yaml` command to expose store-backed capsule YAML contract through CLI.

## New Test Coverage (Milestone 30)
- Capsule lifecycle CLI e2e validating YAML export includes expected capsule metadata fields.

## Additional Work (Milestone 31)
- Added `nexumctl cutover rollback` command to explicitly disable capability flags.
- Rollback output now returns machine-readable confirmation payload (`capability`, `flag`, `rolled_back`).

## New Test Coverage (Milestone 31)
- Cutover CLI e2e validating rollback disables only the targeted capability flag.

## Additional Work (Milestone 32)
- Added `nexumctl stead dispatch-batch` command for multi-event ingest.
- Batch dispatch reuses single-event restore semantics and returns per-event success/failure entries without aborting the whole batch.

## New Test Coverage (Milestone 32)
- Stead CLI e2e validating mixed batch outcomes with deterministic per-event result payloads.
