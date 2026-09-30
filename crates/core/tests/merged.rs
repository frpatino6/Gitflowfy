use gitflowfy_core::fixtures::build_merged;
use std::env;
use std::fs;
use std::process::Command;

#[test]
fn test_merged_fixture() {
    let tmp = env::temp_dir().join(format!("gitflowfy_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let repo_path = tmp.join("merged");
    build_merged(&repo_path).unwrap();

    // Verify merge commit has two parents
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["rev-parse", "HEAD^1", "HEAD^2"])
        .output()
        .unwrap();
    let parents = String::from_utf8_lossy(&output.stdout);
    let parent_count = parents.lines().count();
    assert_eq!(parent_count, 2, "merge commit should have 2 parents");

    // Verify branch-a and branch-b exist
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["branch", "-a"])
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("branch-a"));
    assert!(branches.contains("branch-b"));

    // Cleanup
    let _ = fs::remove_dir_all(&tmp);
}