use nexum::tls::TlsCertificateRecord;

#[test]
fn snapshot_tls_record_contract() {
    let record = TlsCertificateRecord {
        domain: "agent.nexum.local".into(),
        cert_path: "/tmp/agent.nexum.local.crt.pem".into(),
        key_path: "/tmp/agent.nexum.local.key.pem".into(),
        fingerprint_sha256: "abc123".into(),
        created_unix_ms: 1,
        expires_unix_ms: 2,
    };

    insta::assert_yaml_snapshot!("tls_record_contract", record);
}
