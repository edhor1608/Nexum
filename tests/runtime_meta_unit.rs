use nexum::{
    capsule::{Capsule, CapsuleMode},
    runtime_meta::{capsule_runtime_env, terminal_process_label},
};

#[test]
fn runtime_env_prefix_contains_capsule_scoped_metadata() {
    let capsule = Capsule::new("cap-meta-1", "Meta Capsule", CapsuleMode::HostDefault, 12);
    let env = capsule_runtime_env(&capsule);

    assert_eq!(env.get("NEXUM_CAPSULE_ID"), Some(&"cap-meta-1".to_string()));
    assert_eq!(
        env.get("NEXUM_CAPSULE_DOMAIN"),
        Some(&"meta-capsule.nexum.local".to_string())
    );
    assert_eq!(env.get("NEXUM_CAPSULE_WORKSPACE"), Some(&"12".to_string()));
}

#[test]
fn terminal_process_label_includes_capsule_id() {
    let label = terminal_process_label("cap-meta-1");
    assert_eq!(label, "nexum-terminal-cap-meta-1");
}
