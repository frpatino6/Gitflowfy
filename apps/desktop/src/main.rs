use gitflowfy_core::git::{GitCommand, Invocation, AuditLog, AuditRecord, get_ahead_behind};
use gitflowfy_core::state::{detect_repo_state, RepoState as CoreRepoState};
use gitflowfy_core::graph::{BuildConfig, format::Graph};
use gitflowfy_core::git::remote::RemoteRefusal;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RepoPath {
    path: String,
}

#[derive(Deserialize)]
struct GitCommandArgs {
    repo: String,
    args: Vec<String>,
}

#[derive(Deserialize)]
struct DiffCompare {
    repo: String,
    baseline: String,
    candidate: String,
}

#[derive(Serialize)]
struct RepoInfo {
    id: String,
    path: String,
    name: String,
    state: String,
    current_branch: String,
    ahead: i32,
    behind: i32,
    stashes: i32,
}

#[derive(Serialize)]
struct RepoState {
    state: String,
    files: Vec<String>,
}

type AuditLogState = Arc<Mutex<Option<AuditLog>>>;

#[tauri::command]
async fn open_repo(path: String) -> Result<RepoInfo, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let cmd = GitCommand::new(&path, ["rev-parse", "--abbrev-ref", "HEAD"]);
    let result = cmd.run().map_err(|e| e.to_string())?;

    let branch = if result.exit_code == 0 {
        String::from_utf8_lossy(&result.stdout).trim().to_string()
    } else {
        "detached".to_string()
    };

    let ahead_behind = get_ahead_behind(&path).unwrap_or((0, 0));

    Ok(RepoInfo {
        id: uuid::Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        name,
        state: "clean".to_string(),
        current_branch: branch,
        ahead: ahead_behind.0,
        behind: ahead_behind.1,
        stashes: 0,
    })
}

#[tauri::command]
async fn git_command(args: GitCommandArgs) -> Result<Invocation, String> {
    let repo_path = PathBuf::from(&args.repo);
    if !repo_path.exists() {
        return Err("Repository path does not exist".to_string());
    }

    if let Some(first_arg) = args.args.first() {
        if RemoteRefusal::check(first_arg).is_err() {
            return Err(format!("Remote operation '{}' is refused", first_arg));
        }
    }

    let cmd = GitCommand::new(&args.repo, &args.args);
    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
async fn audit_log(repo: String) -> Result<Vec<AuditRecord>, String> {
    let repo_path = PathBuf::from(repo);
    let log = AuditLog::open(&repo_path).map_err(|e| e.to_string())?;
    log.get_records().map_err(|e| e.to_string())
}

#[tauri::command]
async fn audit_clear(repo: String) -> Result<(), String> {
    let repo_path = PathBuf::from(repo);
    let log = AuditLog::open(&repo_path).map_err(|e| e.to_string())?;
    log.truncate().map_err(|e| e.to_string())
}

#[tauri::command]
async fn graph_load(repo: String) -> Result<Vec<u8>, String> {
    let repo_path = PathBuf::from(repo);
    let graph_path = repo_path.join(".git").join("gitflowfy").join("graph.bin");
    let graph = Graph::read_from_file(&graph_path).map_err(|e| e.to_string())?;

    let mut bytes = Vec::new();
    graph.write_to_writer(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

#[tauri::command]
async fn graph_build(repo: String, output: String) -> Result<(), String> {
    let repo_path = PathBuf::from(repo);
    let output_path = PathBuf::from(output);

    let config = BuildConfig::new(repo_path, output_path);
    gitflowfy_core::graph::build_graph(config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn state_detect(repo: String) -> Result<RepoState, String> {
    let repo_path = PathBuf::from(repo);
    let state = detect_repo_state(&repo_path).map_err(|e| e.to_string())?;

    Ok(RepoState {
        state: state.as_str().to_string(),
        files: state.recovery_actions().iter().map(|(a, _)| a.to_string()).collect(),
    })
}

#[tauri::command]
async fn state_recover(repo: String, action: String) -> Result<Invocation, String> {
    let repo_path = PathBuf::from(repo);
    let cmd = match action.as_str() {
        "rebase-continue" => GitCommand::new(&repo_path, ["rebase", "--continue"]),
        "rebase-abort" => GitCommand::new(&repo_path, ["rebase", "--abort"]),
        "merge-continue" => GitCommand::new(&repo_path, ["merge", "--continue"]),
        "merge-abort" => GitCommand::new(&repo_path, ["merge", "--abort"]),
        "cherry-pick-continue" => GitCommand::new(&repo_path, ["cherry-pick", "--continue"]),
        "cherry-pick-abort" => GitCommand::new(&repo_path, ["cherry-pick", "--abort"]),
        "revert-continue" => GitCommand::new(&repo_path, ["revert", "--continue"]),
        "revert-abort" => GitCommand::new(&repo_path, ["revert", "--abort"]),
        "bisect-good" => GitCommand::new(&repo_path, ["bisect", "good"]),
        "bisect-bad" => GitCommand::new(&repo_path, ["bisect", "bad"]),
        "bisect-skip" => GitCommand::new(&repo_path, ["bisect", "skip"]),
        "bisect-reset" => GitCommand::new(&repo_path, ["bisect", "reset"]),
        "unshallow" => GitCommand::new(&repo_path, ["fetch", "--unshallow"]),
        "sparse-disable" => GitCommand::new(&repo_path, ["sparse-checkout", "disable"]),
        "submodule-update" => GitCommand::new(&repo_path, ["submodule", "update", "--init", "--recursive"]),
        _ => return Err("Unknown action".to_string()),
    };

    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
async fn dialog_open() -> Result<String, String> {
    Ok(String::new())
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(None::<AuditLog>))
        .invoke_handler(tauri::generate_handler![
            open_repo,
            git_command,
            audit_log,
            audit_clear,
            graph_load,
            graph_build,
            state_detect,
            state_recover,
            dialog_open,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}