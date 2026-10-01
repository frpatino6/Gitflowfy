use gitflowfy_core::fixtures::build_orphan;
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
fn test_orphan_read_only() {
    let repo1 = unique_tmp("orphan_ro1");
    let repo2 = unique_tmp("orphan_ro2");
    let _ = gitflowfy_core::fixtures::build_orphan(&repo1);
    let _ = gitflowfy_core::fixtures::build_orphan(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["rev-parse", "HEAD"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "read-only rev-parse: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_orphan_mutating() {
    let repo1 = unique_tmp("orphan_mut1");
    let repo2 = unique_tmp("orphan_mut2");
    let _ = gitflowfy_core::fixtures::build_orphan(&repo1);
    let _ = gitflowfy_core::fixtures::build_orphan(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["checkout", "orphan-1"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "mutating checkout: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_orphan_failing() {
    let repo1 = unique_tmp("orphan_fail1");
    let repo2 = unique_tmp("orphan_fail2");
    let _ = gitflowfy_core::fixtures::build_orphan(&repo1);
    let _ = gitflowfy_core::fixtures::build_orphan(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["merge", "nonexistent"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "failing merge: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}