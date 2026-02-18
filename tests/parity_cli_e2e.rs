use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn nexumctl_can_compare_shadow_parity_from_json_inputs() {
    let primary =
        r#"{"capsule_id":"cap-cli-parity","step_count":6,"duration_ms":4000,"attention_priority":"active"}"#;
    let candidate =
        r#"{"capsule_id":"cap-cli-parity","step_count":5,"duration_ms":5000,"attention_priority":"active"}"#;

    let output = Command::new(assert_cmd::cargo::cargo_bin!("nexumctl"))
        .arg("parity")
        .arg("compare")
        .arg("--primary-json")
        .arg(primary)
        .arg("--candidate-json")
        .arg(candidate)
        .output()
        .unwrap();

    assert!(output.status.success());
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["matches"], Value::Bool(false));
    assert!(payload["parity_score"].is_number());
    assert!(
        payload["mismatches"]
            .as_array()
            .unwrap()
            .iter()
            .any(|mismatch| mismatch.as_str().unwrap().contains("step_count mismatch"))
    );
}

#[test]
fn nexumctl_can_compare_shadow_parity_from_files() {
    let dir = tempdir().unwrap();
    let primary_file = dir.path().join("primary.json");
    let candidate_file = dir.path().join("candidate.json");
    std::fs::write(
        &primary_file,
        r#"{"capsule_id":"cap-cli-parity-file","step_count":3,"duration_ms":2000,"attention_priority":"active"}"#,
    )
    .unwrap();
    std::fs::write(
        &candidate_file,
        r#"{"capsule_id":"cap-cli-parity-file","step_count":3,"duration_ms":2100,"attention_priority":"active"}"#,
    )
    .unwrap();

    let output = Command::new(assert_cmd::cargo::cargo_bin!("nexumctl"))
        .arg("parity")
        .arg("compare")
        .arg("--primary-file")
        .arg(&primary_file)
        .arg("--candidate-file")
        .arg(&candidate_file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let payload: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["matches"], Value::Bool(true));
    assert_eq!(payload["parity_score"], Value::from(1.0));
}
