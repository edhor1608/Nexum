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
