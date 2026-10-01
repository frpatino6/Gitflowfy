use gitflowfy_core::fixtures::build_path_edge;
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
fn test_path_edge_read_only() {
    let repo1 = unique_tmp("path_edge_ro1");
    let repo2 = unique_tmp("path_edge_ro2");
    let _ = gitflowfy_core::fixtures::build_path_edge(&repo1);
    let _ = gitflowfy_core::fixtures::build_path_edge(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["status"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "read-only status path-edge: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_path_edge_mutating() {
    let repo1 = unique_tmp("path_edge_mut1");
    let repo2 = unique_tmp("path_edge_mut2");
    let _ = gitflowfy_core::fixtures::build_path_edge(&repo1);
    let _ = gitflowfy_core::fixtures::build_path_edge(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["commit", "--allow-empty", "-m", "test"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "mutating commit path-edge: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_path_edge_failing() {
    let repo1 = unique_tmp("path_edge_fail1");
    let repo2 = unique_tmp("path_edge_fail2");
    let _ = gitflowfy_core::fixtures::build_path_edge(&repo1);
    let _ = gitflowfy_core::fixtures::build_path_edge(&repo2);

    let diff = gitflowfy_core::differential::run_and_compare(&repo1, &repo2, &["checkout", "nonexistent"]).unwrap();
    assert!(diff.observable_diffs.is_empty(), "failing checkout path-edge: {:?}", diff.observable_diffs);

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}