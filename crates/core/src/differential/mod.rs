use std::path::Path;
use std::process::Command;
use std::error::Error;

mod observables;
use observables::{Observables, HeadState, collect_observables, compare_observables};

use crate::git::{GitCommand, Invocation};

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub observable_diffs: Vec<String>,
    pub stdout_diffs: Vec<String>,
    pub stderr_diffs: Vec<String>,
}

pub fn run_through_surface(repo: &Path, args: &[&str]) -> Result<Invocation, Box<dyn std::error::Error>> {
    let cmd = GitCommand::new(repo, args.iter().map(|s| s.to_string()));
    cmd.run().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

pub fn run_through_git(repo: &Path, args: &[&str]) -> Result<Invocation, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("GIT_PAGER", "")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C");
    
    let output = cmd.output()?;
    Ok(Invocation {
        argv: args.iter().map(|s| s.to_string()).collect(),
        exit_code: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
        duration: std::time::Duration::from_millis(0),
    })
}

pub fn run_and_compare(
    left_repo: &Path,
    right_repo: &Path,
    args: &[&str],
) -> Result<DiffResult, Box<dyn std::error::Error>> {
    let left_surface = run_through_surface(left_repo, args)?;
    let right_raw = run_through_git(right_repo, args)?;
    
    let left_obs = observables::collect_observables(left_repo)?;
    let right_obs = observables::collect_observables(right_repo)?;
    
    let observable_diffs = observables::compare_observables(&left_obs, &right_obs);
    
    let left_stdout = normalize_output(&left_surface.stdout, left_repo);
    let right_stdout = normalize_output(&right_raw.stdout, right_repo);
    let stdout_diffs = if left_stdout != right_stdout {
        vec!["stdout".to_string()]
    } else {
        vec![]
    };
    
    let left_stderr = normalize_output(&left_surface.stderr, left_repo);
    let right_stderr = normalize_output(&right_raw.stderr, right_repo);
    let stderr_diffs = if left_stderr != right_stderr {
        vec!["stderr".to_string()]
    } else {
        vec![]
    };
    
    Ok(DiffResult {
        observable_diffs,
        stdout_diffs,
        stderr_diffs,
    })
}

fn normalize_output(bytes: &[u8], repo: &Path) -> String {
    let s = String::from_utf8_lossy(bytes).to_string();
    let repo_str = repo.to_string_lossy();
    let repo_canon = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf()).to_string_lossy().to_string();
    let mut s = s.replace(&*repo_str, "<REPO>");
    s = s.replace(&*repo_canon, "<REPO>");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detects_mutation() {
        // This would require two actual repos - skipping for unit test
    }
}