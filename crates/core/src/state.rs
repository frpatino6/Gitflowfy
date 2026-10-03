use crate::differential::{Observables, collect_observables};
use crate::error::GitflowError;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepoState {
    Clean,
    Empty,
    DetachedHead,
    MergeConflict,
    RebaseInProgress,
    CherryPickInProgress,
    RevertInProgress,
    BisectInProgress,
    PartialClone,
    SparseCheckout,
    SubmoduleConflict,
    MissingReflog,
    ReftableBackend,
}

impl RepoState {
    pub fn as_str(&self) -> &'static str {
        match self {
            RepoState::Clean => "clean",
            RepoState::Empty => "empty",
            RepoState::DetachedHead => "detached-head",
            RepoState::MergeConflict => "merge-conflict",
            RepoState::RebaseInProgress => "rebase-in-progress",
            RepoState::CherryPickInProgress => "cherry-pick-in-progress",
            RepoState::RevertInProgress => "revert-in-progress",
            RepoState::BisectInProgress => "bisect-in-progress",
            RepoState::PartialClone => "partial-clone",
            RepoState::SparseCheckout => "sparse-checkout",
            RepoState::SubmoduleConflict => "submodule-conflict",
            RepoState::MissingReflog => "missing-reflog",
            RepoState::ReftableBackend => "reftable-backend",
        }
    }

    pub fn recovery_actions(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            RepoState::RebaseInProgress => vec![
                ("rebase-continue", "git rebase --continue"),
                ("rebase-abort", "git rebase --abort"),
            ],
            RepoState::MergeConflict => vec![
                ("merge-continue", "git merge --continue"),
                ("merge-abort", "git merge --abort"),
            ],
            RepoState::CherryPickInProgress => vec![
                ("cherry-pick-continue", "git cherry-pick --continue"),
                ("cherry-pick-abort", "git cherry-pick --abort"),
            ],
            RepoState::RevertInProgress => vec![
                ("revert-continue", "git revert --continue"),
                ("revert-abort", "git revert --abort"),
            ],
            RepoState::BisectInProgress => vec![
                ("bisect-good", "git bisect good"),
                ("bisect-bad", "git bisect bad"),
                ("bisect-skip", "git bisect skip"),
                ("bisect-reset", "git bisect reset"),
            ],
            RepoState::PartialClone => vec![
                ("unshallow", "git fetch --unshallow"),
            ],
            RepoState::SparseCheckout => vec![
                ("sparse-disable", "git sparse-checkout disable"),
            ],
            RepoState::SubmoduleConflict => vec![
                ("submodule-update", "git submodule update --init --recursive"),
            ],
            _ => vec![],
        }
    }
}

pub fn detect_repo_state(repo: &Path) -> Result<RepoState, GitflowError> {
    let obs = collect_observables(repo)?;

    // Check rebase in progress
    if repo.join(".git/rebase-merge").exists() || repo.join(".git/rebase-apply").exists() {
        return Ok(RepoState::RebaseInProgress);
    }

    // Check merge conflict
    if repo.join(".git/MERGE_HEAD").exists() {
        if obs.index.contains("U ") {
            return Ok(RepoState::MergeConflict);
        }
        return Ok(RepoState::RebaseInProgress); // Could be merge in progress
    }

    // Check cherry-pick in progress
    if repo.join(".git/CHERRY_PICK_HEAD").exists() {
        return Ok(RepoState::CherryPickInProgress);
    }

    // Check revert in progress
    if repo.join(".git/REVERT_HEAD").exists() {
        return Ok(RepoState::RevertInProgress);
    }

    // Check bisect in progress
    if repo.join(".git/BISECT_LOG").exists() {
        return Ok(RepoState::BisectInProgress);
    }

    // Check empty repo
    if obs.head.rev_parse_exit != 0 {
        return Ok(RepoState::Empty);
    }

    // Check detached HEAD
    if obs.head.symbolic_ref_exit != 0 {
        return Ok(RepoState::DetachedHead);
    }

    // Check partial clone
    if is_partial_clone(repo) {
        return Ok(RepoState::PartialClone);
    }

    // Check sparse checkout
    if is_sparse_checkout(repo) {
        return Ok(RepoState::SparseCheckout);
    }

    // Check reftable backend
    if is_reftable_backend(repo) {
        return Ok(RepoState::ReftableBackend);
    }

    // Check submodule conflict
    if has_submodule_conflict(repo) {
        return Ok(RepoState::SubmoduleConflict);
    }

    // Check missing reflog
    if !repo.join(".git/logs").exists() {
        return Ok(RepoState::MissingReflog);
    }

    // Check merge conflict via index
    if obs.index.contains("U ") {
        return Ok(RepoState::MergeConflict);
    }

    Ok(RepoState::Clean)
}

fn is_partial_clone(repo: &Path) -> bool {
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(["config", "--get", "remote.origin.promisor"])
        .output();
    if let Ok(out) = output {
        if out.status.success() && !out.stdout.is_empty() {
            return true;
        }
    }
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(["config", "--get", "core.partialclonefilter"])
        .output();
    if let Ok(out) = output {
        if out.status.success() && !out.stdout.is_empty() {
            return true;
        }
    }
    false
}

fn is_sparse_checkout(repo: &Path) -> bool {
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(["config", "--get", "core.sparseCheckout"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout);
            let trimmed = val.trim();
            return trimmed == "true";
        }
    }
    false
}

fn is_reftable_backend(repo: &Path) -> bool {
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(["config", "--get", "extensions.refStorage"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let val = String::from_utf8_lossy(&out.stdout);
            let trimmed = val.trim();
            return trimmed == "reftable";
        }
    }
    false
}

fn has_submodule_conflict(repo: &Path) -> bool {
    if !repo.join(".gitmodules").exists() {
        return false;
    }
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(["submodule", "status"])
        .output();
    if let Ok(out) = output {
        let status = String::from_utf8_lossy(&out.stdout);
        return status.lines().any(|line| line.starts_with('U') || line.starts_with('+'));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::process::Command;

    fn init_repo(path: &Path) {
        Command::new("git").args(["init"]).current_dir(path).output().unwrap();
        Command::new("git").args(["config", "user.email", "test@test"]).current_dir(path).output().unwrap();
        Command::new("git").args(["config", "user.name", "Test"]).current_dir(path).output().unwrap();
    }

    #[test]
    fn test_clean_repo() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        init_repo(&repo);
        fs::write(repo.join("file.txt"), "content").unwrap();
        Command::new("git").args(["add", "."]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "initial"]).current_dir(&repo).output().unwrap();
        
        let state = detect_repo_state(&repo).unwrap();
        assert_eq!(state, RepoState::Clean);
    }

    #[test]
    fn test_empty_repo() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        init_repo(&repo);
        
        let state = detect_repo_state(&repo).unwrap();
        assert_eq!(state, RepoState::Empty);
    }

    #[test]
    fn test_detached_head() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        init_repo(&repo);
        fs::write(repo.join("file.txt"), "content").unwrap();
        Command::new("git").args(["add", "."]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "initial"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["checkout", "--detach"]).current_dir(&repo).output().unwrap();
        
        let state = detect_repo_state(&repo).unwrap();
        assert_eq!(state, RepoState::DetachedHead);
    }
}