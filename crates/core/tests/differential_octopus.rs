use gitflowfy_core::fixtures::build_octopus;
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
fn test_octopus_read_only() {
    let repo1 = unique_tmp("octopus_ro1");
    let repo2 = unique_tmp("octopus_ro2");
    let _ = gitflowfy_core::fixtures::build_octopus(&repo1);
    let _ = gitflowfy_core::fixtures::build_octopus(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["branch", "-a"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "read-only branch: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_octopus_mutating() {
    let repo1 = unique_tmp("octopus_mut1");
    let repo2 = unique_tmp("octopus_mut2");
    let _ = gitflowfy_core::fixtures::build_octopus(&repo1);
    let _ = gitflowfy_core::fixtures::build_octopus(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["tag", "test-tag"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "mutating tag: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_octopus_failing() {
    let repo1 = unique_tmp("octopus_fail1");
    let repo2 = unique_tmp("octopus_fail2");
    let _ = gitflowfy_core::fixtures::build_octopus(&repo1);
    let _ = gitflowfy_core::fixtures::build_octopus(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["reset", "nonexistent"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "failing reset: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}