use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::error::GitflowError;
use crate::git::remote::RemoteRefusal;

#[derive(Debug, Clone)]
pub struct GitCommand {
    pub repo: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub argv: Vec<String>,
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub duration: Duration,
}

impl GitCommand {
    pub fn new(repo: impl AsRef<Path>, args: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self {
            repo: repo.as_ref().to_path_buf(),
            args: args.into_iter().map(|s| s.as_ref().to_string()).collect(),
        }
    }

    pub fn run(&self) -> Result<Invocation, GitflowError> {
        if let Some(verb) = self.args.first() {
            RemoteRefusal::check(verb)?;
        }

        let start = Instant::now();

        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo)
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("GIT_PAGER", "")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C");

        let output = cmd.output().map_err(|e| GitflowError::SpawnFailed {
            cmd: format!("git {}", self.args.join(" ")),
            source: e,
        })?;

        let duration = start.elapsed();

        Ok(Invocation {
            argv: self.args.clone(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            duration,
        })
    }
}