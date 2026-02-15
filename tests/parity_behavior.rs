use nexum::shadow::{ExecutionResult, compare_execution};

#[test]
fn parity_report_is_full_match_when_outputs_align() {
    let primary = ExecutionResult {
        capsule_id: "cap-parity-1".into(),
        step_count: 6,
        duration_ms: 4200,
        attention_priority: "active".into(),
    };
    let candidate = primary.clone();

    let report = compare_execution(&primary, &candidate);
    assert!(report.matches);
    assert!(report.mismatches.is_empty());
    assert_eq!(report.parity_score, 1.0);
}

#[test]
fn parity_report_lists_specific_differences() {
    let primary = ExecutionResult {
        capsule_id: "cap-parity-2".into(),
        step_count: 6,
        duration_ms: 4200,
        attention_priority: "active".into(),
    };
    let candidate = ExecutionResult {
        capsule_id: "cap-parity-2".into(),
        step_count: 5,
        duration_ms: 6800,
        attention_priority: "passive".into(),
    };

    let report = compare_execution(&primary, &candidate);
    assert!(!report.matches);
    assert!(report.mismatches.iter().any(|m| m.contains("step_count")));
    assert!(report.mismatches.iter().any(|m| m.contains("duration_ms")));
    assert!(
        report
            .mismatches
            .iter()
            .any(|m| m.contains("attention_priority"))
    );
    assert!(report.parity_score < 1.0);
}
