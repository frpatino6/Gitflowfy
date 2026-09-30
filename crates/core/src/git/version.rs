use std::process::Command;
use crate::error::GitflowError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitVersion {
    pub raw: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl GitVersion {
    pub fn parse(output: &str) -> Result<Self, GitflowError> {
        let line = output.lines().next().ok_or_else(|| {
            GitflowError::VersionParse("empty output".to_string())
        })?;
        let rest = line.strip_prefix("git version ").ok_or_else(|| {
            GitflowError::VersionParse(line.to_string())
        })?;
        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() < 2 {
            return Err(GitflowError::VersionParse(rest.to_string()));
        }
        let major = parts[0].parse().map_err(|_| GitflowError::VersionParse(rest.to_string()))?;
        let minor = parts[1].parse().map_err(|_| GitflowError::VersionParse(rest.to_string()))?;
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        Ok(Self {
            raw: output.trim().to_string(),
            major,
            minor,
            patch,
        })
    }
}

pub fn git_version() -> Result<GitVersion, GitflowError> {
    let output = Command::new("git")
        .arg("version")
        .output()
        .map_err(|_| GitflowError::GitNotFound)?;
    if !output.status.success() {
        return Err(GitflowError::GitNotFound);
    }
    GitVersion::parse(std::str::from_utf8(&output.stdout).unwrap_or(""))
}

pub fn validate_version(found: &GitVersion, expected: &GitVersion) -> Result<(), GitflowError> {
    if found != expected {
        Err(GitflowError::VersionMismatch {
            expected: expected.raw.clone(),
            found: found.raw.clone(),
        })
    } else {
        Ok(())
    }
}