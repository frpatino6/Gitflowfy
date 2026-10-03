use gitflowfy_core::fixtures::build_merged;
use gitflowfy_core::differential::{run_and_compare, DiffResult};
use std::env;
use std::fs;

fn unique_tmp(suffix: &str) -> std::path::PathBuf {
    let base = env::temp_dir().join(format!("gitflowfy_diff_{}_{}", std::process::id(), suffix));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn test_merged_read_only() {
    let repo1 = unique_tmp("merged_ro1");
    let repo2 = unique_tmp("merged_ro2");
    let _ = gitflowfy_core::fixtures::build_merged(&repo1);
    let _ = gitflowfy_core::fixtures::build_merged(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["log", "--oneline", "-5"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "read-only log: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_merged_mutating() {
    let repo1 = unique_tmp("merged_mut1");
    let repo2 = unique_tmp("merged_mut2");
    let _ = gitflowfy_core::fixtures::build_merged(&repo1);
    let _ = gitflowfy_core::fixtures::build_merged(&repo2);

    // Verify repos are identical before mutation
    let diff_before = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["status"]).unwrap();
    assert!(diff_before.observable_diffs.is_empty(), "repos should be identical before mutation");

    // Apply same mutation to both repos
    let _ = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["commit", "--allow-empty", "-m", "test"]).unwrap();

    // After same mutation, repos should still be structurally identical
    // (they will have different commit OIDs due to timestamps, but structure should match)
    let diff_after = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["status"]).unwrap();
    // Allow refs/head/reflog to differ due to different commit OIDs/timestamps
    let structural_diffs: Vec<_> = diff_after.observable_diffs.iter()
        .filter(|d| !["refs", "head", "reflog"].contains(&d.as_str()))
        .collect();
    assert!(structural_diffs.is_empty(), "structural diffs after mutation: {:?}", structural_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_merged_failing() {
    let repo1 = unique_tmp("merged_fail1");
    let repo2 = unique_tmp("merged_fail2");
    let _ = gitflowfy_core::fixtures::build_merged(&repo1);
    let _ = gitflowfy_core::fixtures::build_merged(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["merge", "nonexistent"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "failing merge: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}