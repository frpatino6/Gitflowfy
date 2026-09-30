use gitflowfy_core::cli::{parse_args, Command};
use gitflowfy_core::error::GitflowError;
use gitflowfy_core::git::{AuditLog, AuditRecord, GitCommand};
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::Duration;

fn main() {
    let parsed = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let result = match parsed.command {
        Command::Git { repo, args } => run_git(&repo, &args, parsed.json),
        Command::Audit { repo } => run_audit(&repo, parsed.json),
        Command::Diff { baseline, candidate } => run_diff(&baseline, &candidate, parsed.json),
        Command::Fixtures { shape, path } => run_fixtures(&shape, &path, parsed.json),
        Command::Version => run_version(parsed.json),
    };

    match result {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run_git(repo: &std::path::Path, argv: &[String], json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = GitCommand::new(repo, argv);
    let inv = cmd.run()?;
    
    let log = AuditLog::open(repo)?;
    let record = AuditRecord {
        cwd: repo.to_string_lossy().to_string(),
        argv: argv.to_vec(),
        exit_code: inv.exit_code,
        duration_ms: inv.duration.as_millis() as u64,
    };
    log.append(&record)?;
    log.truncate_reported()?;

    if json {
        let out = serde_json::json!({
            "exit_code": inv.exit_code,
            "stdout": String::from_utf8_lossy(&inv.stdout),
            "stderr": String::from_utf8_lossy(&inv.stderr),
            "duration_ms": inv.duration.as_millis(),
            "audit": record,
        });
        println!("{}", serde_json::to_string(&out)?);
    } else {
        if !inv.stdout.is_empty() {
            io::stdout().write_all(&inv.stdout)?;
        }
        if !inv.stderr.is_empty() {
            io::stderr().write_all(&inv.stderr)?;
        }
    }
    
    if inv.exit_code != 0 {
        return Err(format!("git exited with code {}", inv.exit_code).into());
    }
    Ok(())
}

fn run_audit(repo: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let log = AuditLog::open(repo)?;
    let records = log.read_all()?;
    
    if json {
        println!("{}", serde_json::to_string(&records)?);
    } else {
        for r in records {
            println!("{} {} {} {}", r.cwd, r.argv.join(" "), r.exit_code, r.duration_ms);
        }
    }
    Ok(())
}

fn run_diff(baseline: &Path, candidate: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    if json {
        println!("{}", serde_json::to_string(&serde_json::json!({"status": "not implemented"}))?);
    } else {
        println!("diff not implemented");
    }
    Ok(())
}

fn run_fixtures(shape: &str, path: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    if json {
        println!("{}", serde_json::to_string(&serde_json::json!({"status": "not implemented"}))?);
    } else {
        println!("fixtures build {} not implemented", shape);
    }
    Ok(())
}

fn run_version(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let version = gitflowfy_core::git::git_version()?;
    if json {
        println!("{}", serde_json::to_string(&serde_json::json!({
            "tool": "0.1.0",
            "git": version.raw,
        }))?);
    } else {
        println!("gitflowfy 0.1.0");
        println!("git {}", version.raw);
    }
    Ok(())
}