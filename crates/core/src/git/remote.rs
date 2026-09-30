use crate::error::GitflowError;

/// Network verbs that are refused before any spawn.
/// This is a first-party list; it is necessarily incomplete.
/// See Complexity Tracking in plan.md.
pub const REFUSED_VERBS: &[&str] = &[
    "fetch", "push", "pull", "clone", "remote", "ls-remote",
    "send-pack", "receive-pack", "upload-pack", "upload-archive",
    "credential", "credential-cache", "credential-store",
];

#[derive(Debug, Clone, Copy)]
pub struct RemoteRefusal;

impl RemoteRefusal {
    pub fn check(verb: &str) -> Result<(), GitflowError> {
        if REFUSED_VERBS.contains(&verb) {
            Err(GitflowError::RemoteRefused { verb: verb.to_string() })
        } else {
            Ok(())
        }
    }
}