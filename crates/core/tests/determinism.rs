use gitflowfy_core::fixtures::{build_linear, build_merged, build_octopus, build_orphan, build_detached, build_empty, build_path_edge};
use std::env;
use std::fs;
use std::process::Command;

fn get_head_oid(repo: &std::path::Path) -> String {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_all_oids(repo: &std::path::Path) -> Vec<String> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(["for-each-ref", "--format=%(objectname)", "refs/heads/"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .collect()
}

fn unique_tmp(suffix: &str) -> std::path::PathBuf {
    let base = env::temp_dir().join(format!("gitflowfy_det_{}_{}", std::process::id(), suffix));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn test_linear_determinism() {
    let repo1 = unique_tmp("linear1");
    let repo2 = unique_tmp("linear2");

    gitflowfy_core::fixtures::build_linear(&repo1, 5).unwrap();
    gitflowfy_core::fixtures::build_linear(&repo2, 5).unwrap();

    let oids1 = get_all_oids(&repo1);
    let oids2 = get_all_oids(&repo2);
    assert_eq!(oids1, oids2, "linear fixture OIDs must be identical across builds");

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_merged_determinism() {
    let repo1 = unique_tmp("merged1");
    let repo2 = unique_tmp("merged2");

    gitflowfy_core::fixtures::build_merged(&repo1).unwrap();
    gitflowfy_core::fixtures::build_merged(&repo2).unwrap();

    let oids1 = get_all_oids(&repo1);
    let oids2 = get_all_oids(&repo2);
    assert_eq!(oids1, oids2, "merged fixture OIDs must be identical across builds");

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_octopus_determinism() {
    let repo1 = unique_tmp("octopus1");
    let repo2 = unique_tmp("octopus2");

    gitflowfy_core::fixtures::build_octopus(&repo1).unwrap();
    gitflowfy_core::fixtures::build_octopus(&repo2).unwrap();

    let oids1 = get_all_oids(&repo1);
    let oids2 = get_all_oids(&repo2);
    assert_eq!(oids1, oids2, "octopus fixture OIDs must be identical across builds");

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_orphan_determinism() {
    let repo1 = unique_tmp("orphan1");
    let repo2 = unique_tmp("orphan2");

    gitflowfy_core::fixtures::build_orphan(&repo1).unwrap();
    gitflowfy_core::fixtures::build_orphan(&repo2).unwrap();

    let oids1 = get_all_oids(&repo1);
    let oids2 = get_all_oids(&repo2);
    assert_eq!(oids1, oids2, "orphan fixture OIDs must be identical across builds");

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_detached_determinism() {
    let repo1 = unique_tmp("detached1");
    let repo2 = unique_tmp("detached2");

    gitflowfy_core::fixtures::build_detached(&repo1).unwrap();
    gitflowfy_core::fixtures::build_detached(&repo2).unwrap();

    let oid1 = get_head_oid(&repo1);
    let oid2 = get_head_oid(&repo2);
    assert_eq!(oid1, oid2, "detached fixture HEAD OID must be identical across builds");

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_empty_determinism() {
    let repo1 = unique_tmp("empty1");
    let repo2 = unique_tmp("empty2");

    gitflowfy_core::fixtures::build_empty(&repo1).unwrap();
    gitflowfy_core::fixtures::build_empty(&repo2).unwrap();

    let oids1 = get_all_oids(&repo1);
    let oids2 = get_all_oids(&repo2);
    assert_eq!(oids1, oids2, "empty fixture OIDs must be identical (both empty)");

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}

#[test]
fn test_path_edge_determinism() {
    let repo1 = unique_tmp("path_edge1");
    let repo2 = unique_tmp("path_edge2");

    gitflowfy_core::fixtures::build_path_edge(&repo1).unwrap();
    gitflowfy_core::fixtures::build_path_edge(&repo2).unwrap();

    let oids1 = get_all_oids(&repo1);
    let oids2 = get_all_oids(&repo2);
    assert_eq!(oids1, oids2, "path_edge fixture OIDs must be identical across builds");

    let _ = fs::remove_dir_all(&repo1);
    let _ = fs::remove_dir_all(&repo2);
}