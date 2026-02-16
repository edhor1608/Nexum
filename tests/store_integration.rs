use nexum::{
    capsule::{Capsule, CapsuleMode},
    store::CapsuleStore,
};
use tempfile::tempdir;

#[test]
fn sqlite_store_persists_capsules_across_reopen() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");

    {
        let mut store = CapsuleStore::open(&db).unwrap();
        store
            .upsert(Capsule::new(
                "cap-store-1",
                "Search Core",
                CapsuleMode::HostDefault,
                4,
            ))
            .unwrap();
        store
            .upsert(Capsule::new(
                "cap-store-2",
                "Auth Core",
                CapsuleMode::IsolatedNixShell,
                7,
            ))
            .unwrap();
    }

    let store = CapsuleStore::open(&db).unwrap();
    let listed = store.list().unwrap();

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].capsule_id, "cap-store-1");
    assert_eq!(listed[1].capsule_id, "cap-store-2");
}

#[test]
fn renaming_display_name_keeps_slug_stable_in_store() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");

    let mut store = CapsuleStore::open(&db).unwrap();
    let mut capsule = Capsule::new("cap-store-3", "Billing API", CapsuleMode::HostDefault, 2);
    store.upsert(capsule.clone()).unwrap();

    capsule.rename_display_name("Billing API V2");
    store.upsert(capsule).unwrap();

    let loaded = store.get("cap-store-3").unwrap().unwrap();
    assert_eq!(loaded.slug, "billing-api");
    assert_eq!(loaded.display_name, "Billing API V2");
}

#[test]
fn store_supports_lifecycle_state_transition() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");

    let mut store = CapsuleStore::open(&db).unwrap();
    store
        .upsert(Capsule::new(
            "cap-store-4",
            "Stateful Capsule",
            CapsuleMode::HostDefault,
            3,
        ))
        .unwrap();

    store
        .transition_state("cap-store-4", nexum::capsule::CapsuleState::Degraded)
        .unwrap();

    let loaded = store.get("cap-store-4").unwrap().unwrap();
    assert_eq!(loaded.state, nexum::capsule::CapsuleState::Degraded);
}

#[test]
fn store_allocates_stable_capsule_ports_and_releases_them() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");

    let mut store = CapsuleStore::open(&db).unwrap();
    store
        .upsert(Capsule::new(
            "cap-store-ports",
            "Ports Capsule",
            CapsuleMode::HostDefault,
            5,
        ))
        .unwrap();

    let first = store
        .allocate_port("cap-store-ports", 6100, 6103)
        .unwrap()
        .unwrap();
    let second = store
        .allocate_port("cap-store-ports", 6100, 6103)
        .unwrap()
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(first, 6100);

    let released = store.release_ports("cap-store-ports").unwrap();
    assert_eq!(released, 1);

    let reassigned = store
        .allocate_port("cap-store-other", 6100, 6103)
        .unwrap()
        .unwrap();
    assert_eq!(reassigned, 6100);
}

#[test]
fn store_persists_capsule_repo_path_updates() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");

    let mut store = CapsuleStore::open(&db).unwrap();
    store
        .upsert(
            Capsule::new("cap-store-repo", "Repo Capsule", CapsuleMode::HostDefault, 6)
                .with_repo_path("/workspace/repo-capsule"),
        )
        .unwrap();

    let loaded = store.get("cap-store-repo").unwrap().unwrap();
    assert_eq!(loaded.repo_path, "/workspace/repo-capsule");
}
