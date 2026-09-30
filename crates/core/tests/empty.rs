use gitflowfy_core::fixtures::build_empty;
use std::env;
use std::process::Command;

#[test]
fn test_empty_fixture() {
    let tmp = env::temp_dir().join(format!("gitflowfy_test_{}", std::process::id()));
    std::fs::remove_dir_all(&tmp).ok();
    let repo_path = tmp.join("empty");
    std::fs::create_dir_all(&repo_path).unwrap();
    gitflowfy_core::fixtures::build_empty(&repo_path).unwrap();

    // Verify HEAD is unborn (rev-parse exits 128)
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(128), "rev-parse HEAD should exit 128 on unborn HEAD");

    // Verify symbolic-ref exits 0 naming a non-existent branch
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()
        .unwrap();
    assert!(output.status.success(), "symbolic-ref should exit 0 on unborn HEAD");
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!branch.is_empty(), "symbolic-ref should name a branch");

    // Cleanup
    std::fs::remove_dir_all(&tmp).ok();
}