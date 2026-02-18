use nexum::cutover::{Capability, CutoverInput, evaluate_cutover};

#[test]
fn snapshot_cutover_decision_contract() {
    let decision = evaluate_cutover(&CutoverInput {
        capability: Capability::Restore,
        parity_score: 0.94,
        min_parity_score: 0.95,
        critical_events: 0,
        max_critical_events: 0,
        shadow_mode_enabled: true,
    });

    insta::assert_yaml_snapshot!("cutover_decision_contract", decision);
}
