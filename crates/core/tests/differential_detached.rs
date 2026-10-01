use gitflowfy_core::fixtures::build_detached;
use gitflowfy_core::differential::run_and_compare;
use std::env;
use std::fs;

fn unique_tmp(suffix: &str) -> std::path::PathBuf {
    let base = env::temp_dir().join(format!("gitflowfy_diff_{}_{}", std::process::id(), suffix));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn test_detached_read_only() {
    let repo1 = unique_tmp("detached_ro1");
    let repo2 = unique_tmp("detached_ro2");
    let _ = gitflowfy_core::fixtures::build_detached(&repo1);
    let _ = gitflowfy_core::fixtures::build_detached(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["symbolic-ref", "-q", "HEAD"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "read-only symbolic-ref: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_detached_mutating() {
    let repo1 = unique_tmp("detached_mut1");
    let repo2 = unique_tmp("detached_mut2");
    let _ = gitflowfy_core::fixtures::build_detached(&repo1);
    let _ = gitflowfy_core::fixtures::build_detached(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["commit", "--allow-empty", "-m", "on detached"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "mutating commit on detached: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_detached_failing() {
    let repo1 = unique_tmp("detached_fail1");
    let repo2 = unique_tmp("detached_fail2");
    let _ = gitflowfy_core::fixtures::build_detached(&repo1);
    let _ = gitflowfy_core::fixtures::build_detached(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["checkout", "nonexistent"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "failing checkout: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}