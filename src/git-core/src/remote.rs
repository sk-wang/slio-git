//! Remote operations for git-core

use crate::error::GitError;
use crate::process::git_command;
use crate::repository::Repository;
use git2::{
    Config, Cred, CredentialHelper, Error as Git2Error, FetchOptions, PushOptions, RemoteCallbacks,
};
use log::info;

/// A Git remote
#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

fn remote_url_uses_ssh(url: &str) -> bool {
    if url.starts_with("ssh://") {
        return true;
    }

    if url.contains("://") {
        return false;
    }

    let mut parts = url.splitn(2, ':');
    let Some(left) = parts.next() else {
        return false;
    };

    parts.next().is_some() && left.contains('@')
}

fn resolve_auth_username(
    config: &Config,
    url: &str,
    explicit_username: Option<&str>,
    username_from_url: Option<&str>,
) -> Option<String> {
    explicit_username
        .filter(|username| !username.is_empty())
        .map(str::to_string)
        .or_else(|| username_from_url.map(str::to_string))
        .or_else(|| {
            let mut helper = CredentialHelper::new(url);
            helper.config(config);
            helper.username.clone()
        })
}

fn build_remote_callbacks(
    config: Config,
    credentials: Option<(&str, &str)>,
) -> RemoteCallbacks<'static> {
    let explicit_username = credentials
        .map(|(username, _)| username.trim().to_string())
        .filter(|username| !username.is_empty());
    let explicit_password =
        credentials.and_then(|(_, password)| (!password.is_empty()).then(|| password.to_string()));

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed_types| {
        if allowed_types.is_user_pass_plaintext() {
            if let (Some(username), Some(password)) =
                (explicit_username.as_deref(), explicit_password.as_deref())
            {
                return Cred::userpass_plaintext(username, password);
            }
        }

        let auth_username =
            resolve_auth_username(&config, url, explicit_username.as_deref(), username_from_url);

        if allowed_types.is_username() {
            if let Some(username) = auth_username.as_deref() {
                return Cred::username(username);
            }
        }

        if allowed_types.is_ssh_key() {
            if let Some(username) = auth_username.as_deref() {
                // 1. Try SSH agent first
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }

                // 2. Try common SSH key files from ~/.ssh/
                let ssh_dir = dirs_next::home_dir()
                    .map(|h| h.join(".ssh"))
                    .unwrap_or_default();
                let key_names = [
                    "id_ed25519",
                    "id_rsa",
                    "id_ecdsa",
                    "id_dsa",
                ];
                for key_name in &key_names {
                    let private_key = ssh_dir.join(key_name);
                    if private_key.exists() {
                        let public_key = ssh_dir.join(format!("{key_name}.pub"));
                        let pub_path = public_key.exists().then_some(public_key.as_path());
                        if let Ok(cred) =
                            Cred::ssh_key(username, pub_path, &private_key, None)
                        {
                            return Ok(cred);
                        }
                    }
                }
            }
        }

        if allowed_types.is_user_pass_plaintext() {
            if let Ok(cred) =
                Cred::credential_helper(&config, url, explicit_username.as_deref().or(username_from_url))
            {
                return Ok(cred);
            }
        }

        if allowed_types.is_default() {
            if let Ok(cred) = Cred::default() {
                return Ok(cred);
            }
        }

        Err(Git2Error::from_str(
            "failed to resolve remote credentials from manual input, ssh-agent, or git credential helper",
        ))
    });

    callbacks
}

fn remote_url(repo: &Repository, remote_name: &str) -> Result<String, GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let remote = repo_lock
        .find_remote(remote_name)
        .map_err(|e| GitError::RemoteFailed {
            remote: remote_name.to_string(),
            details: e.to_string(),
        })?;

    Ok(remote.url().unwrap_or("").to_string())
}

fn run_git_remote_command(
    repo: &Repository,
    operation: &str,
    remote_name: &str,
    args: &[&str],
) -> Result<(), GitError> {
    let output = git_command()
        .args(args)
        .current_dir(repo.command_cwd())
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: format!("Failed to execute git {operation}: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let details = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };

        return Err(GitError::RemoteFailed {
            remote: remote_name.to_string(),
            details: format!("git {operation} failed: {details}"),
        });
    }

    Ok(())
}

/// List all remotes
pub fn list_remotes(repo: &Repository) -> Result<Vec<RemoteInfo>, GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let mut remotes = Vec::new();

    let remote_names = repo_lock.remotes().map_err(|e| GitError::OperationFailed {
        operation: "list_remotes".to_string(),
        details: e.to_string(),
    })?;

    for i in 0..remote_names.len() {
        if let Some(name) = remote_names.get(i) {
            if let Ok(remote) = repo_lock.find_remote(name) {
                let url = remote.url().unwrap_or("").to_string();
                remotes.push(RemoteInfo {
                    name: name.to_string(),
                    url,
                });
            }
        }
    }

    Ok(remotes)
}

/// List remotes that are relevant to the current branch workflow.
///
/// If the current branch already tracks an upstream remote, keep the result
/// focused on that remote so the UI can stay anchored to the mainline sync
/// target. When no upstream is configured, fall back to all remotes.
pub fn list_branch_scoped_remotes(repo: &Repository) -> Result<Vec<RemoteInfo>, GitError> {
    let remotes = list_remotes(repo)?;
    let Some(preferred_remote) = repo.current_upstream_remote() else {
        return Ok(remotes);
    };

    let filtered = remotes
        .iter()
        .filter(|remote| remote.name == preferred_remote)
        .cloned()
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        Ok(remotes)
    } else {
        Ok(filtered)
    }
}

/// Fetch from a remote
pub fn fetch(
    repo: &Repository,
    remote_name: &str,
    credentials: Option<(&str, &str)>,
) -> Result<(), GitError> {
    info!("Fetching from remote '{}'", remote_name);

    let remote_url = remote_url(repo, remote_name)?;
    if remote_url_uses_ssh(&remote_url) {
        info!(
            "Using system git fetch for SSH remote '{}' ({})",
            remote_name, remote_url
        );
        return run_git_remote_command(repo, "fetch", remote_name, &["fetch", remote_name]);
    }

    let repo_lock = repo.inner.write().unwrap();
    let config = repo_lock.config().map_err(|e| GitError::RemoteFailed {
        remote: remote_name.to_string(),
        details: e.to_string(),
    })?;
    let mut remote = repo_lock
        .find_remote(remote_name)
        .map_err(|e| GitError::RemoteFailed {
            remote: remote_name.to_string(),
            details: e.to_string(),
        })?;

    let mut callbacks = build_remote_callbacks(config, credentials);

    callbacks.transfer_progress(|progress| {
        info!(
            "Fetch progress: {}/{} objects",
            progress.received_objects(),
            progress.total_objects()
        );
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let refspecs = ["refs/heads/*:refs/remotes/*"];
    remote
        .fetch(&refspecs, Some(&mut fetch_options), None)
        .map_err(|e| GitError::RemoteFailed {
            remote: remote_name.to_string(),
            details: e.to_string(),
        })?;

    info!("Fetch completed successfully");
    Ok(())
}

/// Push to a remote
pub fn push(
    repo: &Repository,
    remote_name: &str,
    branch_name: &str,
    credentials: Option<(&str, &str)>,
) -> Result<(), GitError> {
    info!(
        "Pushing branch '{}' to remote '{}'",
        branch_name, remote_name
    );

    let refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");

    // Try libgit2 first (handles SSH keys from agent + ~/.ssh/ + credential helpers)
    let libgit2_result = (|| -> Result<(), GitError> {
        let repo_lock = repo.inner.write().unwrap();
        let config = repo_lock.config().map_err(|e| GitError::RemoteFailed {
            remote: remote_name.to_string(),
            details: e.to_string(),
        })?;
        let mut remote =
            repo_lock
                .find_remote(remote_name)
                .map_err(|e| GitError::RemoteFailed {
                    remote: remote_name.to_string(),
                    details: e.to_string(),
                })?;

        let mut callbacks = build_remote_callbacks(config, credentials);
        callbacks.push_update_reference(|refname, msg| {
            info!("Push update: {} - {:?}", refname, msg);
            Ok(())
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut push_options))
            .map_err(|e| GitError::RemoteFailed {
                remote: remote_name.to_string(),
                details: e.to_string(),
            })?;

        info!("Push completed successfully via libgit2");
        Ok(())
    })();

    if libgit2_result.is_ok() {
        return libgit2_result;
    }

    // Fallback to system git (handles edge cases libgit2 can't)
    info!(
        "libgit2 push failed ({}), falling back to system git",
        libgit2_result.as_ref().unwrap_err()
    );
    run_git_remote_command(repo, "push", remote_name, &["push", remote_name, &refspec])
}

/// Force push with --force-with-lease semantics
pub fn force_push(repo: &Repository, remote_name: &str, branch_name: &str) -> Result<(), GitError> {
    info!(
        "Force pushing branch '{}' to remote '{}' (--force-with-lease)",
        branch_name, remote_name
    );

    let repo_path = repo.command_cwd();

    let output = git_command()
        .args(["push", "--force-with-lease", remote_name, branch_name])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "force_push".to_string(),
            details: format!("Failed to execute git push --force-with-lease: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::RemoteFailed {
            remote: remote_name.to_string(),
            details: format!(
                "git push --force-with-lease failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    info!("Force push completed successfully");
    Ok(())
}

/// Pull from a remote (fetch + merge)
pub fn pull(
    repo: &Repository,
    remote_name: &str,
    branch_name: &str,
    credentials: Option<(&str, &str)>,
) -> Result<(), GitError> {
    info!(
        "Pulling from remote '{}' branch '{}'",
        remote_name, branch_name
    );

    let repo_path = repo.command_cwd();

    // First fetch from the remote
    fetch(repo, remote_name, credentials)?;

    // Then merge the remote branch into the current branch
    let refspec = format!("{}/{}", remote_name, branch_name);

    // Use git merge to merge the fetched branch
    let output = git_command()
        .args(["merge", &refspec])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "pull".to_string(),
            details: format!("Failed to execute git merge: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "pull".to_string(),
            details: format!(
                "git merge failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    info!("Pull completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{remote_url_uses_ssh, resolve_auth_username};
    use git2::Config;
    use std::fs;
    use tempfile::{tempdir, TempDir};

    fn config_with_username(username: &str) -> (TempDir, Config) {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("gitconfig");
        fs::write(
            &config_path,
            format!("[credential]\n\tusername = {username}\n"),
        )
        .unwrap();

        let config = Config::open(&config_path).unwrap();
        (temp_dir, config)
    }

    #[test]
    fn resolve_auth_username_prefers_explicit_username() {
        let (_temp_dir, config) = config_with_username("saved-user");

        let username = resolve_auth_username(
            &config,
            "https://example.com/repo.git",
            Some("manual-user"),
            Some("remote-user"),
        );

        assert_eq!(username.as_deref(), Some("manual-user"));
    }

    #[test]
    fn resolve_auth_username_falls_back_to_git_credential_username() {
        let (_temp_dir, config) = config_with_username("saved-user");

        let username = resolve_auth_username(&config, "https://example.com/repo.git", None, None);

        assert_eq!(username.as_deref(), Some("saved-user"));
    }

    #[test]
    fn remote_url_uses_ssh_detects_common_ssh_forms() {
        assert!(remote_url_uses_ssh("git@codeup.aliyun.com:group/repo.git"));
        assert!(remote_url_uses_ssh(
            "ssh://git@codeup.aliyun.com/group/repo.git"
        ));
        assert!(!remote_url_uses_ssh(
            "https://codeup.aliyun.com/group/repo.git"
        ));
    }
}
