use gitflowfy_core::fixtures::build_orphan;
use std::env;
use std::process::Command;

#[test]
fn test_orphan_fixture() {
    let tmp = env::temp_dir().join(format!("gitflowfy_test_{}", std::process::id()));
    std::fs::remove_dir_all(&tmp).ok();
    let repo_path = tmp.join("orphan");
    std::fs::create_dir_all(&repo_path).unwrap();
    gitflowfy_core::fixtures::build_orphan(&repo_path).unwrap();

    // Verify two orphan branches exist with no common ancestor
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["merge-base", "orphan-1", "orphan-2"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "orphan branches should have no merge base");

    // Cleanup
    std::fs::remove_dir_all(&tmp).ok();
}