use gitflowfy_core::fixtures::build_linear;
use std::fs;
use std::path::Path;
use std::env;

#[test]
fn test_linear_fixture() {
    let tmp = env::temp_dir().join(format!("gitflowfy_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let repo_path = tmp.join("linear");
    build_linear(&repo_path, 5).unwrap();

    // Verify repo exists and has 5 commits
    let output = std::process::Command::new("git")
        .current_dir(&repo_path)
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .unwrap();
    let count = String::from_utf8_lossy(&output.stdout).trim().parse::<usize>().unwrap();
    assert_eq!(count, 5, "expected 5 commits");

    let _ = fs::remove_dir_all(&tmp);
}