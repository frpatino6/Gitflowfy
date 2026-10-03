use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

use crate::error::GitflowError;

const LOG_DIR: &str = "gitflowfy";
const LOG_FILE: &str = "invocations.jsonl";
const MAX_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB cap

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub cwd: String,
    pub argv: Vec<String>,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    /// Resolve the git directory by asking git, then append our log file there.
    pub fn open(repo: &Path) -> Result<Self, GitflowError> {
        let git_dir = Self::resolve_git_dir(repo)?;
        let log_dir = git_dir.join(LOG_DIR);
        create_dir_all(&log_dir)?;
        let path = log_dir.join(LOG_FILE);
        Ok(Self { path })
    }

    fn resolve_git_dir(repo: &Path) -> Result<PathBuf, GitflowError> {
        let output = Command::new("git")
            .current_dir(repo)
            .arg("rev-parse")
            .arg("--git-dir")
            .output()
            .map_err(|_| GitflowError::GitNotFound)?;
        if !output.status.success() {
            return Err(GitflowError::GitNotFound);
        }
        let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let p = PathBuf::from(dir);
        if p.is_absolute() {
            Ok(p)
        } else {
            Ok(repo.join(p))
        }
    }

    pub fn append(&self, record: &AuditRecord) -> Result<(), GitflowError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(record)
            .map_err(|e| GitflowError::AuditWrite(std::io::Error::other(e)))?;
        writeln!(file, "{}", line).map_err(GitflowError::AuditWrite)?;
        file.flush().map_err(GitflowError::AuditWrite)?;
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<AuditRecord>, GitflowError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(GitflowError::AuditWrite)?;
            if line.trim().is_empty() {
                continue;
            }
            let rec: AuditRecord = serde_json::from_str(&line)
                .map_err(|e| GitflowError::AuditWrite(std::io::Error::other(e)))?;
            records.push(rec);
        }
        Ok(records)
    }

    /// Enforce size cap. Returns true if truncation occurred.
    pub fn enforce_cap(&self) -> Result<bool, GitflowError> {
        let meta = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(_) => return Ok(false),
        };
        if meta.len() <= MAX_BYTES {
            return Ok(false);
        }
        let records = self.read_all()?;
        let keep = records.len() / 2;
        let to_write = &records[keep..];
        let tmp = self.path.with_extension("jsonl.tmp");
        {
            let mut file = BufWriter::new(File::create(&tmp)?);
            for rec in to_write {
                let line = serde_json::to_string(rec)
                    .map_err(|e| GitflowError::AuditWrite(std::io::Error::other(e)))?;
                writeln!(file, "{}", line).map_err(GitflowError::AuditWrite)?;
            }
            file.flush().map_err(GitflowError::AuditWrite)?;
        }
        std::fs::rename(&tmp, &self.path).map_err(GitflowError::AuditWrite)?;
        Ok(true)
    }

    pub fn truncate_reported(&self) -> Result<bool, GitflowError> {
        let truncated = self.enforce_cap()?;
        Ok(truncated)
    }

    pub fn get_records(&self) -> Result<Vec<AuditRecord>, GitflowError> {
        self.read_all()
    }

    pub fn truncate(&self) -> Result<(), GitflowError> {
        self.enforce_cap().map(|_| ())
    }
}