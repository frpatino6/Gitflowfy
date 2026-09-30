use std::env;
use std::path::PathBuf;

use crate::error::GitflowError;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub command: Command,
    pub json: bool,
}

#[derive(Debug, Clone)]
pub enum Command {
    Git { repo: PathBuf, args: Vec<String> },
    Audit { repo: PathBuf },
    Diff { baseline: PathBuf, candidate: PathBuf },
    Fixtures { shape: String, path: PathBuf },
    Version,
}

pub fn parse_args() -> Result<CliArgs, GitflowError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(GitflowError::SpawnFailed {
            cmd: "gitflowfy".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing subcommand"),
        });
    }

    let mut json = false;
    let mut remaining = Vec::new();
    for arg in &args[1..] {
        if arg == "--json" {
            json = true;
        } else {
            remaining.push(arg.clone());
        }
    }

    if remaining.is_empty() {
        return Err(GitflowError::SpawnFailed {
            cmd: "gitflowfy".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing subcommand"),
        });
    }

    let command = match remaining[0].as_str() {
        "git" => {
            if remaining.len() < 4 || remaining[2] != "--" {
                return Err(GitflowError::SpawnFailed {
                    cmd: "gitflowfy git".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "usage: gitflowfy git <repo> -- <argv...>"),
                });
            }
            Command::Git {
                repo: PathBuf::from(&remaining[1]),
                args: remaining[3..].to_vec(),
            }
        }
        "audit" => {
            if remaining.len() != 2 {
                return Err(GitflowError::SpawnFailed {
                    cmd: "gitflowfy audit".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "usage: gitflowfy audit <repo>"),
                });
            }
            Command::Audit {
                repo: PathBuf::from(&remaining[1]),
            }
        }
        "diff" => {
            if remaining.len() != 3 {
                return Err(GitflowError::SpawnFailed {
                    cmd: "gitflowfy diff".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "usage: gitflowfy diff <baseline> <candidate>"),
                });
            }
            Command::Diff {
                baseline: PathBuf::from(&remaining[1]),
                candidate: PathBuf::from(&remaining[2]),
            }
        }
        "fixtures" => {
            if remaining.len() != 3 {
                return Err(GitflowError::SpawnFailed {
                    cmd: "gitflowfy fixtures".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "usage: gitflowfy fixtures build <shape> <path>"),
                });
            }
            if remaining[1] != "build" {
                return Err(GitflowError::SpawnFailed {
                    cmd: "gitflowfy fixtures".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "unknown fixtures subcommand"),
                });
            }
            Command::Fixtures {
                shape: remaining[2].clone(),
                path: PathBuf::from(&remaining[3]),
            }
        }
        "version" => Command::Version,
        _ => {
            return Err(GitflowError::SpawnFailed {
                cmd: "gitflowfy".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("unknown command: {}", remaining[0])),
            });
        }
    };

    Ok(CliArgs { command, json })
}