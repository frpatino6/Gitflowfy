use std::fmt;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitflowError {
    #[error("git not found in PATH")]
    GitNotFound,

    #[error("git version parse failed: {0}")]
    VersionParse(String),

    #[error("git version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("spawn failed for `{cmd}`: {source}")]
    SpawnFailed {
        cmd: String,
        #[source]
        source: io::Error,
    },

    #[error("git command failed (exit {exit_code}): {stderr}")]
    GitFailed {
        exit_code: i32,
        stderr: String,
    },

    #[error("remote operation refused: {verb}")]
    RemoteRefused { verb: String },

    #[error("audit write failed: {0}")]
    AuditWrite(#[from] io::Error),

    #[error("audit log truncation failed: {0}")]
    AuditTruncation(String),

    #[error("repository state unchanged but expected mutation")]
    NoMutation,

    #[error("concurrent append corrupted audit line")]
    CorruptedAuditLine,
}

impl GitflowError {
    pub fn is_git_not_found(&self) -> bool {
        matches!(self, GitflowError::GitNotFound)
    }
}