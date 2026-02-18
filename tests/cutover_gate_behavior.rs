use nexum::cutover::{Capability, CutoverInput, evaluate_cutover};

#[test]
fn allows_cutover_when_all_gates_pass() {
    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Routing,
        parity_score: 0.98,
        min_parity_score: 0.95,
        critical_events: 0,
        max_critical_events: 0,
        shadow_mode_enabled: true,
    });

    assert!(decision.allowed);
    assert!(decision.reasons.is_empty());
    assert_eq!(
        decision.flag_to_enable,
        Some("routing_control_plane".into())
    );
}

#[test]
fn denies_cutover_when_shadow_mode_disabled() {
    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Restore,
        parity_score: 0.99,
        min_parity_score: 0.95,
        critical_events: 0,
        max_critical_events: 0,
        shadow_mode_enabled: false,
    });

    assert!(!decision.allowed);
    assert!(decision.reasons.iter().any(|r| r.contains("shadow_mode")));
    assert_eq!(decision.flag_to_enable, None);
}

#[test]
fn denies_cutover_when_parity_is_below_threshold() {
    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Attention,
        parity_score: 0.80,
        min_parity_score: 0.95,
        critical_events: 0,
        max_critical_events: 0,
        shadow_mode_enabled: true,
    });

    assert!(!decision.allowed);
    assert!(decision.reasons.iter().any(|r| r.contains("parity")));
}

#[test]
fn denies_cutover_when_critical_events_exceed_limit() {
    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Routing,
        parity_score: 0.99,
        min_parity_score: 0.95,
        critical_events: 3,
        max_critical_events: 0,
        shadow_mode_enabled: true,
    });

    assert!(!decision.allowed);
    assert!(
        decision
            .reasons
            .iter()
            .any(|r| r.contains("critical events"))
    );
}

#[test]
fn denies_cutover_when_parity_is_not_finite() {
    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Routing,
        parity_score: f64::NAN,
        min_parity_score: 0.95,
        critical_events: 0,
        max_critical_events: 0,
        shadow_mode_enabled: true,
    });

    assert!(!decision.allowed);
    assert!(decision.reasons.iter().any(|r| r.contains("finite")));
}

#[test]
fn denies_cutover_when_parity_is_out_of_range() {
    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Routing,
        parity_score: 1.1,
        min_parity_score: 0.95,
        critical_events: 0,
        max_critical_events: 0,
        shadow_mode_enabled: true,
    });

    assert!(!decision.allowed);
    assert!(decision.reasons.iter().any(|r| r.contains("between 0 and 1")));
}
