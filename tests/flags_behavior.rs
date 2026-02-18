use nexum::flags::{CutoverFlags, FlagName};
use tempfile::tempdir;

#[test]
fn defaults_to_shadow_mode_only() {
    let flags = CutoverFlags::default();
    assert!(flags.shadow_mode);
    assert!(!flags.routing_control_plane);
    assert!(!flags.restore_control_plane);
    assert!(!flags.attention_control_plane);
}

#[test]
fn persists_flag_changes_to_local_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("flags.toml");

    let mut flags = CutoverFlags::load_or_default(&path).unwrap();
    flags.set(FlagName::RoutingControlPlane, true);
    flags.set(FlagName::RestoreControlPlane, true);
    flags.save(&path).unwrap();

    let loaded = CutoverFlags::load_or_default(&path).unwrap();
    assert!(loaded.shadow_mode);
    assert!(loaded.routing_control_plane);
    assert!(loaded.restore_control_plane);
    assert!(!loaded.attention_control_plane);
}
