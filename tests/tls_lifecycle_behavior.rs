use nexum::tls::{ensure_self_signed_cert, rotate_if_expiring};
use tempfile::tempdir;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn creates_and_reuses_self_signed_cert_material() {
    let dir = tempdir().unwrap();

    let first = ensure_self_signed_cert(dir.path(), "api.nexum.local", 30).unwrap();
    let second = ensure_self_signed_cert(dir.path(), "api.nexum.local", 30).unwrap();

    assert_eq!(first.fingerprint_sha256, second.fingerprint_sha256);
    assert!(dir.path().join("api.nexum.local.crt.pem").exists());
    assert!(dir.path().join("api.nexum.local.key.pem").exists());
    assert!(dir.path().join("api.nexum.local.meta.json").exists());
}

#[test]
fn rotates_when_expiring_soon() {
    let dir = tempdir().unwrap();

    let first = ensure_self_signed_cert(dir.path(), "ops.nexum.local", 1).unwrap();
    let rotate = rotate_if_expiring(dir.path(), "ops.nexum.local", 2).unwrap();

    assert!(rotate.rotated);
    assert_ne!(first.fingerprint_sha256, rotate.record.fingerprint_sha256);
}

#[test]
fn does_not_rotate_when_not_expiring() {
    let dir = tempdir().unwrap();

    let first = ensure_self_signed_cert(dir.path(), "stable.nexum.local", 30).unwrap();
    let rotate = rotate_if_expiring(dir.path(), "stable.nexum.local", 2).unwrap();

    assert!(!rotate.rotated);
    assert_eq!(first.fingerprint_sha256, rotate.record.fingerprint_sha256);
}

#[test]
fn rotate_preserves_existing_validity_window_policy() {
    let dir = tempdir().unwrap();

    let first = ensure_self_signed_cert(dir.path(), "policy.nexum.local", 7).unwrap();
    let rotate = rotate_if_expiring(dir.path(), "policy.nexum.local", 8).unwrap();

    assert!(rotate.rotated);
    let first_validity_days = (first.expires_unix_ms - first.created_unix_ms) / (24 * 60 * 60 * 1000);
    let rotated_validity_days =
        (rotate.record.expires_unix_ms - rotate.record.created_unix_ms) / (24 * 60 * 60 * 1000);
    assert_eq!(first_validity_days, rotated_validity_days);
}

#[test]
fn rejects_invalid_domain_path_traversal_inputs() {
    let dir = tempdir().unwrap();
    let err = ensure_self_signed_cert(dir.path(), "../escape", 30)
        .unwrap_err()
        .to_string();
    assert!(err.contains("invalid domain"));
}

#[cfg(unix)]
#[test]
fn private_key_written_with_owner_only_permissions() {
    let dir = tempdir().unwrap();
    let record = ensure_self_signed_cert(dir.path(), "secure.nexum.local", 30).unwrap();
    let mode = std::fs::metadata(&record.key_path)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}
