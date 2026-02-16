# Decisions Log

## ADR-IMPL-001
Context:
- Needed a first runnable control-plane slice with enforceable behavior contracts.

Decision:
- Use Rust package with library modules + binaries (`nexumd`, `nexumctl`) and TDD across unit/integration/snapshot/e2e.

Rationale:
- Strong type safety for daemon/control-plane code, fast local execution, and good test tooling.

Consequences:
- Requires Rust toolchain as primary dev dependency for this repo.

## ADR-IMPL-002
Context:
- Routing daemon needed local control surface aligned with locked architecture.

Decision:
- Implement newline-delimited JSON commands over Unix domain socket.

Rationale:
- Simple local-only transport, debuggable, and low overhead for single-host dev OS.

Consequences:
- Schema versioning discipline needed as protocol evolves.

## ADR-IMPL-003
Context:
- Capsule identity and migration controls require durable local storage and reversible toggles.

Decision:
- Persist capsules in SQLite and migration flags in local TOML file.

Rationale:
- SQLite provides transactional durability; TOML flags provide simple offline cutover controls.

Consequences:
- Need migration scripts/schema versioning for future store evolution.

## ADR-IMPL-004
Context:
- D-002 moved to deep niri-centric integration, requiring explicit shell-level planning semantics.

Decision:
- Introduce `shell` and `control_plane` modules that convert restore intent into deterministic niri-oriented command plans.

Rationale:
- Keeps restore logic declarative while giving WM-native execution behavior.

Consequences:
- Command contract stability now matters for future niri adapter implementation.

## ADR-IMPL-005
Context:
- D-008 reframed to internal capability cutovers with shadow validation.

Decision:
- Add `shadow` parity evaluator and `events` SQLite sink, and expose parity/flag operations through `nexumctl`.

Rationale:
- Enables measurable rollout gates and reversible cutovers with local-only operation.

Consequences:
- Requires operational policy thresholds for parity acceptance in later rollout phases.

## ADR-IMPL-006
Context:
- Control-plane plans needed an explicit execution boundary to avoid coupling directly to process spawning and to prepare for real niri integration.

Decision:
- Introduce `NiriAdapter` trait with deterministic `execute_shell_plan` and script rendering surface.

Rationale:
- Enables testing of shell execution semantics independent of runtime adapter choice.

Consequences:
- Future niri runtime adapter can be added without changing restore/control-plane planning contracts.

## ADR-IMPL-007
Context:
- Routing policy requires self-signed per-capsule TLS lifecycle that is local, deterministic, and testable.

Decision:
- Add `tls` module with on-disk cert/key material + metadata and threshold-based rotation API.

Rationale:
- Keeps TLS policy concrete while staying local-first and simple to operate.

Consequences:
- Metadata file integrity is now part of TLS operational correctness.

## ADR-IMPL-008
Context:
- Internal capability cutovers need explicit policy gates tied to parity and runtime health.

Decision:
- Add `cutover` module that evaluates gate criteria and applies capability flags only when all gates pass.

Rationale:
- Makes rollout decisions deterministic, auditable, and reversible.

Consequences:
- Threshold tuning (parity and critical event limits) becomes an operational responsibility.

## ADR-IMPL-009
Context:
- Needed one executable end-to-end control-plane path to validate composition of previously isolated modules.

Decision:
- Add `runflow` module and `nexumctl run restore` command that orchestrate restore planning + routing + TLS + shell rendering + events.

Rationale:
- Provides a concrete, testable integration slice suitable for rollout rehearsals.

Consequences:
- Current runflow uses in-process route registration and should later be wired to daemon-backed routing for production path parity.

## ADR-IMPL-010
Context:
- `nexumd` daemon protocol existed, but operators lacked a first-class CLI interface to exercise and verify it.

Decision:
- Add `nexumctl routing` subcommands that call daemon socket API (`health/register/resolve/remove/list`) and return JSON outcomes.

Rationale:
- Keeps routing operations scriptable and testable without bespoke socket tooling.

Consequences:
- CLI now owns async runtime bridging logic and must stay aligned with routing protocol contract changes.

## ADR-IMPL-011
Context:
- Restore orchestration still used in-process route registration, leaving a parity gap against daemon-driven routing operations.

Decision:
- Add optional daemon routing socket support to `run_restore_flow` and expose it via `nexumctl run restore --routing-socket`.

Rationale:
- Aligns restore path with control-plane-first routing behavior while preserving offline local fallback.

Consequences:
- Runflow now owns dual-path routing logic (daemon-backed and fallback in-process), which must remain behaviorally consistent.

## ADR-IMPL-012
Context:
- v1 acceptance criteria required explicit proof for parallel capsule restore stability and sub-10s restore behavior.

Decision:
- Add dedicated acceptance-level e2e tests for 5 concurrent capsule restores and wall-clock restore completion budget.

Rationale:
- Keeps key product outcomes enforced in CI rather than implicit assumptions from lower-level tests.

Consequences:
- E2E suite runtime grows, but regressions against core v1 outcomes are now caught early.

## ADR-IMPL-013
Context:
- Hybrid identity strategy required profile fallback behavior when domain isolation is insufficient due detected session collision.

Decision:
- Add `identity` policy module and restore CLI/runflow input flag (`--identity-collision`) to switch browser launch from `xdg-open` to dedicated profile command.

Rationale:
- Preserves domain-first default behavior while enabling deterministic fallback path for collision cases.

Consequences:
- Restore command contract now includes collision signaling input, and shell script generation includes policy-specific browser launch substitution.

## ADR-IMPL-014
Context:
- Hybrid isolation policy required deterministic escalation beyond identity collision, including secret-sensitive workflows and explicit user overrides.

Decision:
- Add `isolation` policy module and expose escalation controls in restore CLI (`--high-risk-secret`, `--force-isolated`), with resulting mode surfaced in restore summary.

Rationale:
- Makes isolation behavior explicit, testable, and auditable while preserving host-default as baseline.

Consequences:
- Restore input/output contracts expanded, including new mode signal (`run_mode`) that downstream consumers may rely on.

## ADR-IMPL-015
Context:
- Isolation policy required capsule-scoped runtime metadata and process-label conventions to improve observability and execution traceability.

Decision:
- Add `runtime_meta` module and prepend restore shell scripts with `NEXUM_*` exports plus `NEXUM_PROCESS_LABEL`.

Rationale:
- Makes capsule identity explicit in runtime execution surfaces without changing daemon protocol.

Consequences:
- Shell script output contract changed and is now guarded by snapshot/e2e tests.

## ADR-IMPL-016
Context:
- Capsule specification required explicit lifecycle state transitions and metadata update operations, but implementation only supported create/list.

Decision:
- Add `CapsuleState` to core model, persist it in SQLite with migration-safe schema extension, and expose lifecycle mutations via CLI (`capsule rename`, `capsule set-state`).

Rationale:
- Makes lifecycle management enforceable through tested command paths rather than ad-hoc direct DB edits.

Consequences:
- Capsule serialization and YAML export contracts changed to include `state`.

## ADR-IMPL-017
Context:
- Capsule specification required allocate/release port operations, but implementation only had an in-memory allocator disconnected from persisted capsule operations.

Decision:
- Add persisted `capsule_ports` mapping in SQLite and expose deterministic allocation/release through store + CLI commands.

Rationale:
- Keeps port ownership durable across process restarts while retaining predictable allocation semantics.

Consequences:
- Capsule list output now includes `allocated_ports`, and CLI contract expands with port lifecycle commands.

## ADR-IMPL-018
Context:
- Restore flow specification required degraded continuation when routing is unavailable, but implementation aborted restore on socket-level routing failures.

Decision:
- Treat routing transport failures as degraded restore outcomes (`degraded=true`) while keeping domain-conflict responses as hard failures.

Rationale:
- Preserves user recovery path under transient local routing issues while still preventing unsafe domain ownership conflicts.

Consequences:
- Restore summary contract expanded with degraded fields, and downstream consumers can differentiate soft-degraded vs hard-failed restore outcomes.

## ADR-IMPL-019
Context:
- Migration cutover gates required checking critical runtime regressions, but CLI cutover flow relied on manually supplied critical-event counts.

Decision:
- Add event-driven cutover command (`cutover apply-from-events`) that derives critical counts from runtime event storage per capsule.

Rationale:
- Reduces operator error and aligns cutover gating with observed runtime state.

Consequences:
- Cutover CLI surface now includes an event-store dependent path in addition to manual gate-input path.

## ADR-IMPL-020
Context:
- Capsule specification required persisted repository path metadata per capsule, but capsule contract and storage did not include it.

Decision:
- Add `repo_path` to capsule model, persist it in SQLite (with migration-safe column addition), and expose CLI mutation/query paths.

Rationale:
- Enables restore and supervision flows to carry explicit workspace-root metadata as part of capsule state.

Consequences:
- Capsule serialization and list/export contracts changed to include `repo_path`.

## ADR-IMPL-021
Context:
- Runtime observability needed an operator-facing rollup view, but event inspection required raw per-event reads.

Decision:
- Add `EventStore::summary()` and expose it via `nexumctl events summary --db <path>`.

Rationale:
- Provides a fast supervisor health view (global totals + per-capsule critical counts) without custom ad-hoc queries.

Consequences:
- Events CLI contract expands with a summary endpoint and stable JSON shape for monitoring/tooling consumers.

## ADR-IMPL-022
Context:
- Capsule lifecycle states were persisted and mutable, but restore execution did not drive state transitions in the store.

Decision:
- Add optional restore-store wiring (`--capsule-db`) so runflow transitions capsule state to `restoring` and then finalizes to `ready` or `degraded`.

Rationale:
- Aligns runtime behavior with capsule lifecycle contract and makes restore health visible in persisted control-plane state.

Consequences:
- Restore CLI contract expands with optional capsule DB input, and runflow now has store dependency for lifecycle persistence when enabled.

## ADR-IMPL-023
Context:
- Restore CLI required repeated manual metadata input (`name`, `workspace`, default surface paths) even when capsule state was already persisted.

Decision:
- Add `nexumctl run restore-capsule` to resolve capsule metadata from store and derive default restore surfaces from capsule `repo_path`.

Rationale:
- Moves restore closer to one-action capsule recovery behavior while reducing input drift between persisted capsule metadata and runtime command invocation.

Consequences:
- CLI surface expands with store-driven restore entrypoint and explicit validation when repo metadata is insufficient for default surface derivation.

## ADR-IMPL-024
Context:
- Event observability offered summary and capsule-specific history, but lacked a supervisor-friendly query surface for recent filtered events.

Decision:
- Add filtered event query support in store and expose it via `nexumctl events list`.

Rationale:
- Enables direct runtime triage workflows (critical-only, capsule-scoped, bounded recent lists) without external SQL tooling.

Consequences:
- Events CLI contract expands with list query flags and deterministic newest-first ordering semantics.

## ADR-IMPL-025
Context:
- Stead/OS integration boundary was implicit through manual CLI invocations, with no typed ingress contract for runtime signal dispatch.

Decision:
- Add typed Stead dispatch envelope parsing and expose command `nexumctl stead dispatch`.

Rationale:
- Creates a concrete contract between Stead signal producers and OS restore orchestration while keeping execution testable in CLI/e2e flows.

Consequences:
- CLI surface expands with Stead dispatch entrypoint; invalid envelopes now fail fast with explicit parse errors.

## ADR-IMPL-026
Context:
- Operators had to manually combine capsule state, cutover flags, and runtime events from multiple commands for health triage.

Decision:
- Add unified `nexumctl supervisor status` report command.

Rationale:
- Reduces supervision friction by providing one deterministic machine-readable status payload for control-plane triage.

Consequences:
- CLI surface expands with supervisor health endpoint and per-capsule last-event metadata contract.
