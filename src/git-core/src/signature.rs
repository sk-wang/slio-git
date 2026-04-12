//! GPG/SSH signature verification for git-core

use crate::error::GitError;
use crate::process::git_command;
use crate::repository::Repository;
use log::info;
use std::collections::HashMap;
use std::sync::RwLock;

/// Signature type
#[derive(Debug, Clone, PartialEq)]
pub enum SignatureType {
    Gpg,
    Ssh,
    Unknown,
}

/// GPG/SSH signature verification result for a commit
#[derive(Debug, Clone)]
pub struct SignatureStatus {
    /// Whether the commit has a signature
    pub is_signed: bool,
    /// Whether the signature verified successfully
    pub is_verified: bool,
    /// Name from the signing key
    pub signer_name: Option<String>,
    /// Key fingerprint/ID
    pub key_id: Option<String>,
    /// Signature type
    pub signature_type: SignatureType,
}

impl SignatureStatus {
    /// Create an unsigned status
    pub fn unsigned() -> Self {
        Self {
            is_signed: false,
            is_verified: false,
            signer_name: None,
            key_id: None,
            signature_type: SignatureType::Unknown,
        }
    }
}

/// Cache for signature verification results (commit hash → status)
pub struct SignatureCache {
    cache: RwLock<HashMap<String, SignatureStatus>>,
}

impl SignatureCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, commit_id: &str) -> Option<SignatureStatus> {
        self.cache.read().unwrap().get(commit_id).cloned()
    }

    pub fn insert(&self, commit_id: String, status: SignatureStatus) {
        self.cache.write().unwrap().insert(commit_id, status);
    }
}

impl Default for SignatureCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract and verify the signature of a commit
pub fn verify_commit_signature(
    repo: &Repository,
    commit_id: &str,
) -> Result<SignatureStatus, GitError> {
    info!("Verifying signature for commit: {}", commit_id);

    let repo_lock = repo.inner.read().unwrap();
    let oid = git2::Oid::from_str(commit_id).map_err(|e| GitError::CommitNotFound {
        id: format!("{}: {}", commit_id, e),
    })?;

    let commit = repo_lock
        .find_commit(oid)
        .map_err(|_| GitError::CommitNotFound {
            id: commit_id.to_string(),
        })?;

    // Try to extract signature from commit
    let (signature, signed_data) = match repo_lock.extract_signature(&oid, None) {
        Ok((sig, data)) => (sig, data),
        Err(_) => {
            info!("No signature found for commit {}", commit_id);
            return Ok(SignatureStatus::unsigned());
        }
    };

    let sig_str = signature.as_str().unwrap_or("");
    let _data_str = signed_data.as_str().unwrap_or("");

    // Detect signature type
    let signature_type = if sig_str.contains("-----BEGIN PGP SIGNATURE-----") {
        SignatureType::Gpg
    } else if sig_str.contains("-----BEGIN SSH SIGNATURE-----") {
        SignatureType::Ssh
    } else {
        SignatureType::Unknown
    };

    // Shell out to verify
    let repo_path = repo.command_cwd();
    let output = git_command()
        .args(["verify-commit", "--raw", commit_id])
        .current_dir(&repo_path)
        .output();

    let (is_verified, signer_name, key_id) = match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let verified = stderr.contains("[GNUPG:] GOODSIG")
                || stderr.contains("[GNUPG:] VALIDSIG")
                || out.status.success();

            let signer = stderr
                .lines()
                .find(|l| l.contains("GOODSIG"))
                .and_then(|l| l.split_whitespace().nth(3))
                .map(|s| s.to_string())
                .or_else(|| commit.author().name().map(|n| n.to_string()));

            let kid = stderr
                .lines()
                .find(|l| l.contains("VALIDSIG"))
                .and_then(|l| l.split_whitespace().nth(2))
                .map(|s| s.to_string());

            (verified, signer, kid)
        }
        Err(_) => (false, None, None),
    };

    let status = SignatureStatus {
        is_signed: true,
        is_verified,
        signer_name,
        key_id,
        signature_type,
    };

    info!(
        "Signature verification for {}: signed={}, verified={}",
        commit_id, status.is_signed, status.is_verified
    );

    Ok(status)
}
