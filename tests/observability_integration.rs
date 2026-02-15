use nexum::events::{EventStore, RuntimeEvent};
use tempfile::tempdir;

#[test]
fn event_store_persists_and_lists_runtime_events() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("events.sqlite3");

    let mut store = EventStore::open(&db).unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-obs-1".into(),
            component: "routing".into(),
            level: "info".into(),
            message: "route registered".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-obs-1".into(),
            component: "restore".into(),
            level: "warn".into(),
            message: "browser fallback".into(),
            ts_unix_ms: 1200,
        })
        .unwrap();

    let listed = store.list_for_capsule("cap-obs-1").unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].component, "routing");
    assert_eq!(listed[1].component, "restore");
}
