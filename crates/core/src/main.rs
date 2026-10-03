use gitflowfy_core::cli::{parse_args, Command};
use gitflowfy_core::error::GitflowError;
use gitflowfy_core::git::{AuditLog, AuditRecord, GitCommand, RepoLock};
use gitflowfy_core::differential::{run_and_compare, DiffResult};
use gitflowfy_core::fixtures::{build_linear, build_merged, build_octopus, build_orphan, build_detached, build_empty};
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use ctrlc;

static CANCELLED: AtomicBool = AtomicBool::new(false);

fn main() {
    let cancelled = Arc::new(AtomicBool::new(false));
    let c = cancelled.clone();
    ctrlc::set_handler(move || {
        c.store(true, Ordering::SeqCst);
        CANCELLED.store(true, Ordering::SeqCst);
    }).ok();

    let parsed = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let result = match parsed.command {
        Command::Git { repo, args } => run_git(&repo, &args, parsed.json, cancelled),
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

fn run_git(repo: &std::path::Path, argv: &[String], json: bool, cancelled: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    let _lock = RepoLock::acquire(repo)?;

    if CANCELLED.load(Ordering::SeqCst) || cancelled.load(Ordering::SeqCst) {
        return Err("Operation cancelled".into());
    }

    let cmd = GitCommand::new(repo, argv);
    let inv = cmd.run()?;
    
    if CANCELLED.load(Ordering::SeqCst) || cancelled.load(Ordering::SeqCst) {
        return Err("Operation cancelled after execution".into());
    }

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
    let result: DiffResult = run_and_compare(baseline, candidate, &["status"])?;

    if json {
        println!("{}", serde_json::to_string(&result)?);
    } else {
        if result.observable_diffs.is_empty() && result.stdout_diffs.is_empty() && result.stderr_diffs.is_empty() {
            println!("No differences");
        } else {
            for d in &result.observable_diffs {
                println!("Observable diff: {}", d);
            }
            for d in &result.stdout_diffs {
                println!("Stdout diff: {}", d);
            }
            for d in &result.stderr_diffs {
                println!("Stderr diff: {}", d);
            }
        }
    }
    Ok(())
}

fn run_fixtures(shape: &str, path: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let result = match shape {
        "linear" => build_linear(path, 10),
        "merged" => build_merged(path),
        "octopus" => build_octopus(path),
        "orphan" => build_orphan(path),
        "detached" => build_detached(path),
        "empty" => build_empty(path),
        _ => return Err(format!("Unknown fixture shape: {}", shape).into()),
    };

    result?;

    if json {
        println!("{}", serde_json::to_string(&serde_json::json!({
            "status": "created",
            "shape": shape,
            "path": path.to_string_lossy(),
        }))?);
    } else {
        println!("Created {} fixture at {}", shape, path.display());
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