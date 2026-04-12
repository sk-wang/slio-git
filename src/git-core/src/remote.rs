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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PullOptions<'a> {
    pub branch_name: Option<&'a str>,
    pub rebase: bool,
    pub ff_only: bool,
    pub no_ff: bool,
    pub squash: bool,
    pub force_autocrlf_false: bool,
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

fn configured_upstream_branch(repo: &Repository, remote_name: &str) -> Option<String> {
    let branch_name = repo.current_branch().ok().flatten()?;
    let repo_lock = repo.inner.read().ok()?;
    let config = repo_lock.config().ok()?;
    let configured_remote = config
        .get_string(&format!("branch.{branch_name}.remote"))
        .ok()?;

    if configured_remote != remote_name {
        return None;
    }

    let merge_ref = config
        .get_string(&format!("branch.{branch_name}.merge"))
        .ok()?;
    let upstream_branch = merge_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(merge_ref.as_str());

    (!upstream_branch.is_empty()).then(|| upstream_branch.to_string())
}

fn current_branch(repo: &Repository, operation: &str) -> Result<String, GitError> {
    repo.current_branch()
        .map_err(|error| GitError::OperationFailed {
            operation: operation.to_string(),
            details: error.to_string(),
        })?
        .ok_or_else(|| GitError::OperationFailed {
            operation: operation.to_string(),
            details: "Detached HEAD, cannot pull.".to_string(),
        })
}

fn build_pull_args(
    repo: &Repository,
    remote_name: &str,
    options: PullOptions<'_>,
) -> Result<Vec<String>, GitError> {
    let mut args = Vec::new();
    if options.force_autocrlf_false {
        args.push("-c".to_string());
        args.push("core.autocrlf=false".to_string());
    }
    args.push("pull".to_string());

    if options.rebase {
        args.push("--rebase".to_string());
    } else {
        // Match GUI clients like TortoiseGit by explicitly disabling rebase for
        // the default merge-based pull path instead of inheriting ambient config.
        args.push("--no-rebase".to_string());
    }
    if options.ff_only {
        args.push("--ff-only".to_string());
    }
    if options.no_ff {
        args.push("--no-ff".to_string());
    }
    if options.squash {
        args.push("--squash".to_string());
    }

    let explicit_branch = options.branch_name.map(str::trim).filter(|branch| !branch.is_empty());
    if let Some(branch_name) = explicit_branch {
        args.push(remote_name.to_string());
        args.push(branch_name.to_string());
        return Ok(args);
    }

    if configured_upstream_branch(repo, remote_name).is_some() {
        args.push(remote_name.to_string());
        return Ok(args);
    }

    let branch_name = current_branch(repo, "pull")?;

    args.push(remote_name.to_string());
    args.push(branch_name);
    Ok(args)
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

/// Pull from a remote.
pub fn pull(
    repo: &Repository,
    remote_name: &str,
    branch_name: &str,
    credentials: Option<(&str, &str)>,
) -> Result<(), GitError> {
    pull_with_options(
        repo,
        remote_name,
        PullOptions {
            branch_name: Some(branch_name),
            ..PullOptions::default()
        },
        credentials,
    )
}

/// Pull from a remote using system Git semantics.
pub fn pull_with_options(
    repo: &Repository,
    remote_name: &str,
    options: PullOptions<'_>,
    _credentials: Option<(&str, &str)>,
) -> Result<(), GitError> {
    info!(
        "Pulling from remote '{}' with options {:?}",
        remote_name, options
    );

    let repo_path = repo.command_cwd();
    let args = build_pull_args(repo, remote_name, options)?;
    let output = git_command()
        .args(&args)
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "pull".to_string(),
            details: format!("Failed to execute git pull: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "pull".to_string(),
            details: format!(
                "git pull failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    info!("Pull completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_pull_args, remote_url_uses_ssh, resolve_auth_username, PullOptions};
    use crate::repository::Repository;
    use git2::Config;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
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

    fn git(cwd: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("run git command");
        assert!(
            output.status.success(),
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn create_committed_repo() -> (TempDir, String) {
        let repo_dir = tempdir().unwrap();
        git(repo_dir.path(), &["init"]);
        git(repo_dir.path(), &["config", "user.email", "tests@example.com"]);
        git(repo_dir.path(), &["config", "user.name", "Test User"]);
        fs::write(repo_dir.path().join("tracked.txt"), "base\n").unwrap();
        git(repo_dir.path(), &["add", "tracked.txt"]);
        git(repo_dir.path(), &["commit", "-m", "base"]);
        let branch_name = git(repo_dir.path(), &["rev-parse", "--abbrev-ref", "HEAD"]);
        (repo_dir, branch_name)
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
    fn build_pull_args_uses_configured_upstream_when_branch_is_empty() {
        let (repo_dir, branch_name) = create_committed_repo();
        let remote_dir = tempdir().unwrap();
        git(remote_dir.path(), &["init", "--bare"]);
        git(
            repo_dir.path(),
            &["remote", "add", "origin", &remote_dir.path().display().to_string()],
        );
        git(repo_dir.path(), &["push", "-u", "origin", &branch_name]);

        let repo = Repository::open(repo_dir.path()).unwrap();
        let args = build_pull_args(&repo, "origin", PullOptions::default()).unwrap();

        assert_eq!(args, vec!["pull", "--no-rebase", "origin"]);
    }

    #[test]
    fn build_pull_args_falls_back_to_current_branch_without_upstream() {
        let (repo_dir, branch_name) = create_committed_repo();
        let remote_dir = tempdir().unwrap();
        git(remote_dir.path(), &["init", "--bare"]);
        git(
            repo_dir.path(),
            &["remote", "add", "origin", &remote_dir.path().display().to_string()],
        );

        let repo = Repository::open(repo_dir.path()).unwrap();
        let args = build_pull_args(&repo, "origin", PullOptions::default()).unwrap();

        assert_eq!(args, vec!["pull", "--no-rebase", "origin", &branch_name]);
    }

    #[test]
    fn build_pull_args_passes_explicit_branch_and_strategy_flags() {
        let (repo_dir, _) = create_committed_repo();
        let repo = Repository::open(repo_dir.path()).unwrap();
        let args = build_pull_args(
            &repo,
            "origin",
            PullOptions {
                branch_name: Some("release/main"),
                rebase: true,
                ..PullOptions::default()
            },
        )
        .unwrap();

        assert_eq!(args, vec!["pull", "--rebase", "origin", "release/main"]);
    }

    #[test]
    fn build_pull_args_includes_autocrlf_override_when_requested() {
        let (repo_dir, branch_name) = create_committed_repo();
        let remote_dir = tempdir().unwrap();
        git(remote_dir.path(), &["init", "--bare"]);
        git(
            repo_dir.path(),
            &["remote", "add", "origin", &remote_dir.path().display().to_string()],
        );

        let repo = Repository::open(repo_dir.path()).unwrap();
        let args = build_pull_args(
            &repo,
            "origin",
            PullOptions {
                force_autocrlf_false: true,
                ..PullOptions::default()
            },
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "-c",
                "core.autocrlf=false",
                "pull",
                "--no-rebase",
                "origin",
                &branch_name
            ]
        );
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
