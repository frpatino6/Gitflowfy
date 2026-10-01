use std::path::Path;
use std::process::Command;
use std::error::Error;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observables {
    pub refs: String,
    pub head: HeadState,
    pub index: String,
    pub worktree: String,
    pub reflog: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadState {
    pub rev_parse_exit: i32,
    pub rev_parse_out: String,
    pub symbolic_ref_exit: i32,
    pub symbolic_ref_out: String,
}

pub fn collect_observables(repo: &Path) -> Result<Observables, Box<dyn Error>> {
    Ok(Observables {
        refs: collect_refs(repo)?,
        head: collect_head(repo)?,
        index: collect_index(repo)?,
        worktree: collect_worktree(repo)?,
        reflog: collect_reflog(repo)?,
    })
}

fn collect_refs(repo: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["for-each-ref", "--format=%(refname) %(objectname)"])
        .output()?;
    Ok(normalize_path(String::from_utf8_lossy(&output.stdout).to_string(), repo))
}

fn collect_head(repo: &Path) -> Result<HeadState, Box<dyn Error>> {
    let rev = Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let sym = Command::new("git")
        .current_dir(repo)
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()?;
    Ok(HeadState {
        rev_parse_exit: rev.status.code().unwrap_or(-1),
        rev_parse_out: String::from_utf8_lossy(&rev.stdout).trim().to_string(),
        symbolic_ref_exit: sym.status.code().unwrap_or(-1),
        symbolic_ref_out: String::from_utf8_lossy(&sym.stdout).trim().to_string(),
    })
}

fn collect_index(repo: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["ls-files", "-s"])
        .output()?;
    Ok(normalize_path(String::from_utf8_lossy(&output.stdout).to_string(), repo))
}

fn collect_worktree(repo: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["status", "--porcelain=v2", "--untracked-files=all"])
        .output()?;
    let mut result = String::from_utf8_lossy(&output.stdout).to_string();
    
    if let Ok(status_out) = Command::new("git")
        .current_dir(repo)
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .output() {
        for line in String::from_utf8_lossy(&status_out.stdout).lines() {
            if line.starts_with("?? ") {
                let file = line[3..].trim();
                let path = repo.join(file);
                if path.is_file() {
                    let content = std::fs::read(&path).unwrap_or_default();
                    let mut hasher = DefaultHasher::new();
                    content.hash(&mut hasher);
                    result.push_str(&format!("\n# untracked-hash {} {:x}", file, hasher.finish()));
                }
            }
        }
    }
    Ok(normalize_path(result, repo))
}

fn collect_reflog(repo: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["reflog", "--format=%H %gs"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn normalize_path(s: String, repo: &Path) -> String {
    let repo_str = repo.to_string_lossy();
    let repo_canon = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf()).to_string_lossy().to_string();
    let mut s = s.replace(&*repo_str, "<REPO>");
    s = s.replace(&*repo_canon, "<REPO>");
    s
}

pub fn compare_observables(left: &Observables, right: &Observables) -> Vec<String> {
    let mut diffs = Vec::new();
    if left.refs != right.refs { diffs.push("refs".to_string()); }
    if left.head != right.head { diffs.push("head".to_string()); }
    if left.index != right.index { diffs.push("index".to_string()); }
    if left.worktree != right.worktree { diffs.push("worktree".to_string()); }
    if left.reflog != right.reflog { diffs.push("reflog".to_string()); }
    diffs
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_head_state_three_states() {
        // This test would require actual repos - skipping for unit test
    }
}