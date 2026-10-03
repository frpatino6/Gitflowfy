pub mod format;
pub mod builder;
pub mod diff;

pub use format::{Graph, GraphHeader, ArrayOffsets, GRAPH_MAGIC, GRAPH_VERSION, HEADER_SIZE};
pub use builder::{build_graph, BuildConfig};
pub use diff::{compare_graphs, compare_graph_files, format_graph_diff, validate_graph_against_git, GraphDiff};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::process::Command;

    fn init_test_repo(path: &std::path::Path) {
        Command::new("git").args(["init"]).current_dir(path).output().unwrap();
        Command::new("git").args(["config", "user.email", "test@test"]).current_dir(path).output().unwrap();
        Command::new("git").args(["config", "user.name", "Test"]).current_dir(path).output().unwrap();
    }

    #[test]
    fn test_full_graph_build_and_compare() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        init_test_repo(&repo);
        
        // Create linear history with 5 commits
        for i in 0..5 {
            fs::write(repo.join(format!("file{}.txt", i)), format!("content {}", i)).unwrap();
            Command::new("git").args(["add", "."]).current_dir(&repo).output().unwrap();
            Command::new("git").args(["commit", "-m", &format!("commit {}", i)]).current_dir(&repo).output().unwrap();
        }
        
        let output = dir.path().join("graph.bin");
        let config = builder::BuildConfig::new(&repo, &output);
        builder::build_graph(config).unwrap();
        
        // Load and verify
        let graph = format::Graph::read_from_file(&output).unwrap();
        assert_eq!(graph.header.magic, format::GRAPH_MAGIC);
        assert_eq!(graph.header.version, format::GRAPH_VERSION);
        assert_eq!(graph.header.commit_count, 5);
        assert_eq!(graph.header.edge_count, 4); // 4 edges in linear chain
        
        // Verify CSR structure
        assert_eq!(graph.parent_index.len(), 6); // n+1
        assert_eq!(graph.parent_index[0], 0);
        assert_eq!(graph.parent_index[5], 4); // 4 edges total
        
// Verify CSR edges
        // git log --topo-order gives newest first, so commit i points to commit i+1
        assert_eq!(graph.parent_list.len(), 4);
        for i in 0..4 {
            assert_eq!(graph.parent_list[i], (i + 1) as u32); // Each commit points to previous (i+1 in topo-order)
        }
        
        // Verify lanes assigned
        assert_eq!(graph.lane.len(), 5);
        for lane in &graph.lane {
            assert!(*lane >= 0);
        }
    }
}