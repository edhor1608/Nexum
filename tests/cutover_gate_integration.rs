use nexum::{
    cutover::{Capability, CutoverInput, apply_cutover, evaluate_cutover},
    flags::CutoverFlags,
    shadow::{ExecutionResult, compare_execution},
};

#[test]
fn applies_allowed_cutover_to_flags() {
    let report = compare_execution(
        &ExecutionResult {
            capsule_id: "cap-gate-1".into(),
            step_count: 6,
            duration_ms: 4000,
            attention_priority: "active".into(),
        },
        &ExecutionResult {
            capsule_id: "cap-gate-1".into(),
            step_count: 6,
            duration_ms: 4100,
            attention_priority: "active".into(),
        },
    );

    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Routing,
        parity_score: report.parity_score,
        min_parity_score: 0.95,
        critical_events: 0,
        max_critical_events: 0,
        shadow_mode_enabled: true,
    });

    let mut flags = CutoverFlags::default();
    apply_cutover(&mut flags, &decision, Capability::Routing);

    assert!(flags.routing_control_plane);
    assert!(!flags.restore_control_plane);
    assert!(!flags.attention_control_plane);
}
