use nexum::shadow::{ExecutionResult, compare_execution};

#[test]
fn snapshot_shadow_parity_report_contract() {
    let primary = ExecutionResult {
        capsule_id: "cap-parity-snap".into(),
        step_count: 6,
        duration_ms: 4800,
        attention_priority: "blocking".into(),
    };
    let candidate = ExecutionResult {
        capsule_id: "cap-parity-snap".into(),
        step_count: 6,
        duration_ms: 5100,
        attention_priority: "blocking".into(),
    };

    let report = compare_execution(&primary, &candidate);
    insta::assert_yaml_snapshot!("shadow_parity_report_contract", report);
}
