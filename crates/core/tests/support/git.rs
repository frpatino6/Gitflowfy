use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Test-only git runner. This is NOT product code; it is the helper that Phase 0
/// uses to build fixtures and read observables without depending on the operation surface.
pub struct TestGitRunner {
    repo: PathBuf,
    env: Vec<(String, String)>,
}

impl TestGitRunner {
    pub fn new(repo: impl AsRef<Path>) -> Self {
        Self {
            repo: repo.as_ref().to_path_buf(),
            env: vec![
                ("GIT_PAGER".into(), "".into()),
                ("GIT_TERMINAL_PROMPT".into(), "0".into()),
                ("LC_ALL".into(), "C".into()),
            ],
        }
    }

    pub fn run(&self, args: &[&str]) -> Result<TestGitOutput, std::io::Error> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        let output = cmd.output()?;
        Ok(TestGitOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    pub fn version(&self) -> Result<String, std::io::Error> {
        let out = self.run(&["version"])?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
}

#[derive(Debug)]
pub struct TestGitOutput {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl TestGitOutput {
    pub fn stdout_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }
    pub fn stderr_str(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}