use gitflowfy_core::fixtures::build_octopus;
use std::env;
use std::process::Command;

#[test]
fn test_octopus_fixture() {
    let tmp = env::temp_dir().join(format!("gitflowfy_test_{}", std::process::id()));
    std::fs::remove_dir_all(&tmp).ok();
    let repo_path = tmp.join("octopus");
    std::fs::create_dir_all(&repo_path).unwrap();
    gitflowfy_core::fixtures::build_octopus(&repo_path).unwrap();

    // Verify merge commit has three parents
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["rev-parse", "HEAD^1", "HEAD^2", "HEAD^3"])
        .output()
        .unwrap();
    let parents = String::from_utf8_lossy(&output.stdout);
    let parent_count = parents.lines().count();
    assert_eq!(parent_count, 3, "octopus merge commit should have 3 parents");

    // Cleanup
    std::fs::remove_dir_all(&tmp).ok();
}