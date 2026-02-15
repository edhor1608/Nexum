use std::process::Command;

#[test]
fn nexumctl_can_compare_shadow_parity_from_json_inputs() {
    let nexumctl = assert_cmd::cargo::cargo_bin!("nexumctl");

    let primary =
        r#"{"capsule_id":"cap-cli-parity","step_count":6,"duration_ms":4000,"attention_priority":"active"}"#;
    let candidate =
        r#"{"capsule_id":"cap-cli-parity","step_count":5,"duration_ms":5000,"attention_priority":"active"}"#;

    let output = Command::new(nexumctl)
        .arg("parity")
        .arg("compare")
        .arg("--primary-json")
        .arg(primary)
        .arg("--candidate-json")
        .arg(candidate)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"matches\":false"));
    assert!(stdout.contains("\"parity_score\":"));
    assert!(stdout.contains("step_count"));
}
