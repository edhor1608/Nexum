use nexum::tls::{ensure_self_signed_cert, rotate_if_expiring};
use tempfile::tempdir;

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
