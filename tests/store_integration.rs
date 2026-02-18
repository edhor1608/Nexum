use nexum::{
    capsule::{Capsule, CapsuleMode},
    store::CapsuleStore,
};
use rusqlite::Connection;
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
fn store_rejects_unknown_capsule_mode_values() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("capsules.sqlite3");
    let _ = CapsuleStore::open(&db).unwrap();

    let conn = Connection::open(&db).unwrap();
    conn.execute(
        "INSERT INTO capsules (capsule_id, slug, display_name, mode, workspace) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["cap-bad-mode", "bad-mode", "Bad Mode", "future_mode", 9u16],
    )
    .unwrap();

    let store = CapsuleStore::open(&db).unwrap();
    let err = store.list().unwrap_err().to_string();
    assert!(err.contains("invalid mode: future_mode"));
}
