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

#[test]
fn event_store_lists_recent_events_with_filters_and_limit() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("events.sqlite3");

    let mut store = EventStore::open(&db).unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-obs-a".into(),
            component: "routing".into(),
            level: "info".into(),
            message: "route ready".into(),
            ts_unix_ms: 1000,
        })
        .unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-obs-a".into(),
            component: "restore".into(),
            level: "critical".into(),
            message: "restore failed".into(),
            ts_unix_ms: 1100,
        })
        .unwrap();
    store
        .append(RuntimeEvent {
            capsule_id: "cap-obs-b".into(),
            component: "restore".into(),
            level: "critical".into(),
            message: "another failure".into(),
            ts_unix_ms: 1200,
        })
        .unwrap();

    let listed = store
        .list_recent(Some("cap-obs-a"), Some("critical"), Some(5))
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].capsule_id, "cap-obs-a");
    assert_eq!(listed[0].message, "restore failed");

    let latest = store.list_recent(None, Some("critical"), Some(1)).unwrap();
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].capsule_id, "cap-obs-b");
    assert_eq!(latest[0].ts_unix_ms, 1200);
}
