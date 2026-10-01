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

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["commit", "--allow-empty", "-m", "test"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "mutating commit: {:?}", diff.observable_diffs);

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