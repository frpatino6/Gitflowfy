use gitflowfy_core::fixtures::build_linear;
use gitflowfy_core::differential::{run_and_compare, DiffResult};
use std::env;
use std::fs;
use std::process::Command;

fn unique_tmp(suffix: &str) -> std::path::PathBuf {
    let base = env::temp_dir().join(format!("gitflowfy_diff_{}_{}", std::process::id(), suffix));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn test_linear_read_only() {
    let repo1 = unique_tmp("linear_ro1");
    let repo2 = unique_tmp("linear_ro2");
    let _ = gitflowfy_core::fixtures::build_linear(&repo1, 5);
    let _ = gitflowfy_core::fixtures::build_linear(&repo2, 5);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["status"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "read-only status: {:?}", diff.observable_diffs);
    assert!(diff.stdout_diffs.is_empty());
    assert!(diff.stderr_diffs.is_empty());

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_linear_mutating() {
    let repo1 = unique_tmp("linear_mut1");
    let repo2 = unique_tmp("linear_mut2");
    let _ = gitflowfy_core::fixtures::build_linear(&repo1, 3);
    let _ = gitflowfy_core::fixtures::build_linear(&repo2, 3);

    // Mutating: add a file and commit
    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["commit", "--allow-empty", "-m", "test"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "mutating commit: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_linear_failing() {
    let repo1 = unique_tmp("linear_fail1");
    let repo2 = unique_tmp("linear_fail2");
    let _ = gitflowfy_core::fixtures::build_linear(&repo1, 3);
    let _ = gitflowfy_core::fixtures::build_linear(&repo2, 3);

    // Failing: diff with non-existent commit
    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["diff", "nonexistent"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "failing diff: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}