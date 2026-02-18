use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const DAY_MS: u64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsCertificateRecord {
    pub domain: String,
    pub cert_path: String,
    pub key_path: String,
    pub fingerprint_sha256: String,
    pub created_unix_ms: u64,
    pub expires_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotateOutcome {
    pub rotated: bool,
    pub record: TlsCertificateRecord,
}

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("rcgen: {0}")]
    Rcgen(#[from] rcgen::Error),
}

pub fn ensure_self_signed_cert(
    dir: &Path,
    domain: &str,
    validity_days: u64,
) -> Result<TlsCertificateRecord, TlsError> {
    std::fs::create_dir_all(dir)?;

    if let Some(record) = load_record(dir, domain)? {
        if cert_path(dir, domain).exists() && key_path(dir, domain).exists() {
            return Ok(record);
        }
    }

    generate_and_store(dir, domain, validity_days)
}

pub fn rotate_if_expiring(
    dir: &Path,
    domain: &str,
    threshold_days: u64,
) -> Result<RotateOutcome, TlsError> {
    let current = ensure_self_signed_cert(dir, domain, 30)?;
    let now = now_unix_ms();

    let remaining_days = if current.expires_unix_ms > now {
        (current.expires_unix_ms - now) / DAY_MS
    } else {
        0
    };

    if remaining_days <= threshold_days {
        let rotated = generate_and_store(dir, domain, 30)?;
        return Ok(RotateOutcome {
            rotated: true,
            record: rotated,
        });
    }

    Ok(RotateOutcome {
        rotated: false,
        record: current,
    })
}

fn generate_and_store(
    dir: &Path,
    domain: &str,
    validity_days: u64,
) -> Result<TlsCertificateRecord, TlsError> {
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, domain);

    let mut params = CertificateParams::new(vec![domain.to_string()])?;
    params.distinguished_name = distinguished_name;

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let cert_path = cert_path(dir, domain);
    let key_path = key_path(dir, domain);

    std::fs::write(&cert_path, cert_pem.as_bytes())?;
    std::fs::write(&key_path, key_pem.as_bytes())?;

    let created_unix_ms = now_unix_ms();
    let expires_unix_ms = created_unix_ms.saturating_add(validity_days.saturating_mul(DAY_MS));

    let record = TlsCertificateRecord {
        domain: domain.to_string(),
        cert_path: cert_path.display().to_string(),
        key_path: key_path.display().to_string(),
        fingerprint_sha256: sha256_hex(cert_pem.as_bytes()),
        created_unix_ms,
        expires_unix_ms,
    };

    std::fs::write(meta_path(dir, domain), serde_json::to_vec_pretty(&record)?)?;

    Ok(record)
}

fn load_record(dir: &Path, domain: &str) -> Result<Option<TlsCertificateRecord>, TlsError> {
    let path = meta_path(dir, domain);
    if !path.exists() {
        return Ok(None);
    }

    let bytes = std::fs::read(path)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

fn cert_path(dir: &Path, domain: &str) -> std::path::PathBuf {
    dir.join(format!("{domain}.crt.pem"))
}

fn key_path(dir: &Path, domain: &str) -> std::path::PathBuf {
    dir.join(format!("{domain}.key.pem"))
}

fn meta_path(dir: &Path, domain: &str) -> std::path::PathBuf {
    dir.join(format!("{domain}.meta.json"))
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_millis() as u64
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();

    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}
