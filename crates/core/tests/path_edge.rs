use gitflowfy_core::fixtures::build_path_edge;
use std::env;
use std::fs;
use std::process::Command;

#[test]
fn test_path_edge_fixture() {
    let tmp = env::temp_dir().join(format!("gitflowfy_test_{}", std::process::id()));
    fs::remove_dir_all(&tmp).ok();
    let repo_path = tmp.join("path-edge");
    fs::create_dir_all(&repo_path).unwrap();
    gitflowfy_core::fixtures::build_path_edge(&repo_path).unwrap();

    // Verify the file exists on disk (builder created it)
    let entries: Vec<_> = fs::read_dir(&repo_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .collect();
    let has_file_with_espacios = entries.iter().any(|e| {
        e.file_name().to_string_lossy().contains("espacios")
    });
    assert!(has_file_with_espacios, "file with spaces and non-ASCII should exist on disk");

    // Verify it's tracked by git (at least one file tracked)
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["ls-files"])
        .output()
        .unwrap();
    assert!(output.status.success(), "git ls-files should succeed");
    let files = String::from_utf8_lossy(&output.stdout);
    assert!(!files.trim().is_empty(), "at least one file should be tracked by git");
    assert!(files.contains("espacios"), "file with spaces should be tracked by git");

    // Cleanup
    fs::remove_dir_all(&tmp).ok();
}