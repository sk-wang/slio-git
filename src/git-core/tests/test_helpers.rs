//! Test utilities for git-core
//!
//! Provides reusable helpers for testing Git operations

use git_core::{GitError, Repository};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a temporary directory for testing
pub struct TestRepo {
    pub path: TempDir,
}

impl TestRepo {
    /// Create a new temporary git repository
    pub fn new() -> Result<Self, GitError> {
        let temp_dir = tempfile::tempdir().map_err(|e| GitError::Io(std::io::Error::other(e)))?;
        let _repo = Repository::init(temp_dir.path())?;
        for args in [
            ["config", "user.name", "Codex Test"],
            ["config", "user.email", "codex@example.com"],
        ] {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(temp_dir.path())
                .output()
                .map_err(GitError::Io)?;

            if !output.status.success() {
                return Err(GitError::OperationFailed {
                    operation: "git config".to_string(),
                    details: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            }
        }
        Ok(Self { path: temp_dir })
    }

    /// Create a new temporary directory (not a git repo)
    pub fn empty() -> Result<Self, GitError> {
        let temp_dir = tempfile::tempdir().map_err(|e| GitError::Io(std::io::Error::other(e)))?;
        Ok(Self { path: temp_dir })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        self.path.path()
    }

    /// Write a file to the repository
    pub fn write_file(&self, relative_path: &str, content: &str) -> std::io::Result<()> {
        let path = self.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)
    }

    /// Add a file and create a commit
    pub fn add_and_commit(
        &self,
        relative_path: &str,
        content: &str,
        message: &str,
    ) -> Result<(), GitError> {
        self.write_file(relative_path, content)
            .map_err(GitError::Io)?;

        // Use git commands to stage and commit
        let output = std::process::Command::new("git")
            .args(["add", relative_path])
            .current_dir(self.path())
            .output()
            .map_err(GitError::Io)?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "git add".to_string(),
                details: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let output = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(self.path())
            .output()
            .map_err(GitError::Io)?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "git commit".to_string(),
                details: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new().expect("Failed to create test repository")
    }
}
