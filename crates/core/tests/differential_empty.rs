use gitflowfy_core::fixtures::build_empty;
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
fn test_empty_read_only() {
    let repo1 = unique_tmp("empty_ro1");
    let repo2 = unique_tmp("empty_ro2");
    let _ = gitflowfy_core::fixtures::build_empty(&repo1);
    let _ = gitflowfy_core::fixtures::build_empty(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["status"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "read-only status on empty: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_empty_mutating() {
    let repo1 = unique_tmp("empty_mut1");
    let repo2 = unique_tmp("empty_mut2");
    let _ = gitflowfy_core::fixtures::build_empty(&repo1);
    let _ = gitflowfy_core::fixtures::build_empty(&repo2);

    // On empty repo, initial commit is a mutating operation
    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["commit", "--allow-empty", "-m", "first"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "mutating initial commit: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_empty_failing() {
    let repo1 = unique_tmp("empty_fail1");
    let repo2 = unique_tmp("empty_fail2");
    let _ = gitflowfy_core::fixtures::build_empty(&repo1);
    let _ = gitflowfy_core::fixtures::build_empty(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["diff", "HEAD"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "failing diff on empty: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}