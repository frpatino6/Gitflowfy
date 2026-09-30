use gitflowfy_core::fixtures::build_detached;
use std::env;
use std::process::Command;

#[test]
fn test_detached_fixture() {
    let tmp = env::temp_dir().join(format!("gitflowfy_test_{}", std::process::id()));
    std::fs::remove_dir_all(&tmp).ok();
    let repo_path = tmp.join("detached");
    std::fs::create_dir_all(&repo_path).unwrap();
    gitflowfy_core::fixtures::build_detached(&repo_path).unwrap();

    // Verify HEAD is detached (symbolic-ref exits 1)
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "symbolic-ref should fail on detached HEAD");

    // But rev-parse HEAD should succeed
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(output.status.success(), "rev-parse HEAD should succeed on detached HEAD");

    // Cleanup
    std::fs::remove_dir_all(&tmp).ok();
}