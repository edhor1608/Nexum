use nexum::{
    capsule::{Capsule, CapsuleMode},
    store::CapsuleStore,
};
use tempfile::tempdir;

#[test]
fn snapshot_capsule_yaml_export_contract() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");
    let mut store = CapsuleStore::open(&db).unwrap();

    store
        .upsert(Capsule::new(
            "cap-yaml-1",
            "Planner Service",
            CapsuleMode::HostDefault,
            8,
        ))
        .unwrap();
    store
        .upsert(Capsule::new(
            "cap-yaml-2",
            "Agent Gateway",
            CapsuleMode::IsolatedNixShell,
            9,
        ))
        .unwrap();

    let exported = store.export_yaml().unwrap();
    insta::assert_snapshot!("capsule_yaml_export_contract", exported);
}
