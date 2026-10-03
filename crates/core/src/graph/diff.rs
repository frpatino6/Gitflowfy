use crate::graph::format::{Graph, GraphHeader};
use crate::error::GitflowError;
use std::path::Path;

/// Result of comparing two graphs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphDiff {
    pub header_match: bool,
    pub parent_index_match: bool,
    pub parent_list_match: bool,
    pub commit_time_match: bool,
    pub lane_match: bool,
    pub parent_count_match: bool,
    pub strings_match: bool,
    pub differences: Vec<String>,
}

impl GraphDiff {
    pub fn is_identical(&self) -> bool {
        self.differences.is_empty()
    }
}

/// Compare two graphs byte-for-byte (for differential testing)
pub fn compare_graphs(left: &Graph, right: &Graph) -> GraphDiff {
    let mut diff = GraphDiff {
        header_match: true,
        parent_index_match: true,
        parent_list_match: true,
        commit_time_match: true,
        lane_match: true,
        parent_count_match: true,
        strings_match: true,
        differences: Vec::new(),
    };
    
    // Compare headers
    if left.header.magic != right.header.magic {
        diff.header_match = false;
        diff.differences.push(format!("Magic mismatch: left=0x{:08x}, right=0x{:08x}", left.header.magic, right.header.magic));
    }
    if left.header.version != right.header.version {
        diff.header_match = false;
        diff.differences.push(format!("Version mismatch: left={}, right={}", left.header.version, right.header.version));
    }
    if left.header.commit_count != right.header.commit_count {
        diff.header_match = false;
        diff.differences.push(format!("Commit count mismatch: left={}, right={}", left.header.commit_count, right.header.commit_count));
    }
    if left.header.edge_count != right.header.edge_count {
        diff.header_match = false;
        diff.differences.push(format!("Edge count mismatch: left={}, right={}", left.header.edge_count, right.header.edge_count));
    }
    
    // Compare parent_index
    if left.parent_index != right.parent_index {
        diff.parent_index_match = false;
        diff.differences.push(format!("parent_index differs: len left={}, right={}", left.parent_index.len(), right.parent_index.len()));
    }
    
    // Compare parent_list
    if left.parent_list != right.parent_list {
        diff.parent_list_match = false;
        diff.differences.push(format!("parent_list differs: len left={}, right={}", left.parent_list.len(), right.parent_list.len()));
    }
    
    // Compare commit_time
    if left.commit_time != right.commit_time {
        diff.commit_time_match = false;
        diff.differences.push("commit_time differs".to_string());
    }
    
    // Compare lane
    if left.lane != right.lane {
        diff.lane_match = false;
        diff.differences.push("lane differs".to_string());
    }
    
    // Compare parent_count
    if left.parent_count != right.parent_count {
        diff.parent_count_match = false;
        diff.differences.push("parent_count differs".to_string());
    }
    
    // Compare strings
    if left.strings != right.strings {
        diff.strings_match = false;
        diff.differences.push("strings differ".to_string());
    }
    
    diff
}

/// Compare two graph files on disk
pub fn compare_graph_files<P: AsRef<Path>, Q: AsRef<Path>>(left: P, right: Q) -> Result<GraphDiff, crate::error::GitflowError> {
    let left_graph = crate::graph::format::Graph::read_from_file(left)?;
    let right_graph = crate::graph::format::Graph::read_from_file(right)?;
    Ok(compare_graphs(&left_graph, &right_graph))
}

/// Generate a human-readable diff report
pub fn format_graph_diff(diff: &GraphDiff) -> String {
    let mut report = String::new();
    
    if diff.is_identical() {
        report.push_str("Graphs are identical ✓\n");
        return report;
    }
    
    report.push_str("Graph differences:\n");
    
    if !diff.header_match {
        report.push_str("  ✗ Header mismatch\n");
    }
    if !diff.parent_index_match {
        report.push_str("  ✗ parent_index mismatch\n");
    }
    if !diff.parent_list_match {
        report.push_str("  ✗ parent_list mismatch\n");
    }
    if !diff.commit_time_match {
        report.push_str("  ✗ commit_time mismatch\n");
    }
    if !diff.lane_match {
        report.push_str("  ✗ lane mismatch\n");
    }
    if !diff.parent_count_match {
        report.push_str("  ✗ parent_count mismatch\n");
    }
    if !diff.strings_match {
        report.push_str("  ✗ strings mismatch\n");
    }
    
    for d in &diff.differences {
        report.push_str(&format!("  - {}\n", d));
    }
    
    report
}

/// Compare graph built by our builder vs reference graph (from spike or git)
pub fn validate_graph_against_git<P: AsRef<std::path::Path>>(
    repo: P,
    graph_path: P,
) -> Result<GraphDiff, crate::error::GitflowError> {
    let graph = crate::graph::format::Graph::read_from_file(graph_path)?;
    
    // Generate reference graph using git log (simplified)
    let reference = build_reference_graph(repo)?;
    
    Ok(compare_graphs(&reference, &graph))
}

/// Build reference graph using git commands (for validation)
fn build_reference_graph<P: AsRef<std::path::Path>>(repo: P) -> Result<crate::graph::format::Graph, crate::error::GitflowError> {
    use std::process::Command;
    use std::collections::HashMap;
    
    let output = Command::new("git")
        .current_dir(repo)
        .args([
            "log",
            "--all",
            "--topo-order",
            "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%D",
        ])
        .output()?;
    
    if !output.status.success() {
        return Err(crate::error::GitflowError::SpawnFailed {
            cmd: "git log".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr).to_string(),
            ),
        });
    }
    
    // Parse and build graph (simplified - reuse builder logic)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = super::builder::parse_git_log(&stdout)?;
    
    // Build graph from entries (same logic as builder)
    let n = entries.len() as u32;
    let mut oid_to_index = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        oid_to_index.insert(e.oid.clone(), i as u32);
    }
    
    let mut parent_index = Vec::with_capacity(entries.len() + 1);
    let mut parent_list = Vec::new();
    let mut current_offset = 0u32;
    
    for entry in &entries {
        parent_index.push(current_offset);
        for parent_oid in &entry.parents {
            if let Some(&idx) = oid_to_index.get(parent_oid) {
                parent_list.push(idx);
            }
        }
        current_offset = parent_list.len() as u32;
    }
    parent_index.push(current_offset);
    
    let n = entries.len() as u32;
    let m = parent_list.len() as u32;
    let mut graph = crate::graph::format::Graph::new(n, m);
    graph.parent_index = parent_index;
    graph.parent_list = parent_list;
    
    // Fill data
    for (i, entry) in entries.iter().enumerate() {
        graph.commit_time[i] = entry.author_date;
        graph.parent_count[i] = entry.parents.len().min(3) as u8;
        // Lane assignment would be done here
        // String packing
    }
    
    // Lane assignment
    crate::graph::builder::assign_lanes(&mut graph);
    
    Ok(graph)
}

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
    fn test_compare_identical() {
        use crate::graph::builder::{build_graph, BuildConfig};
use crate::graph::format::Graph;
use crate::graph::diff::compare_graphs;

        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        
        Command::new("git").args(["init"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["config", "user.email", "test@test"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["config", "user.name", "Test"]).current_dir(&repo).output().unwrap();
        
        fs::write(repo.join("file.txt"), "content").unwrap();
        Command::new("git").args(["add", "."]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "initial"]).current_dir(&repo).output().unwrap();
        
        let graph1 = dir.path().join("graph1.bin");
        let graph2 = dir.path().join("graph2.bin");
        
        // Build same graph twice
        let config1 = BuildConfig::new(&repo, &graph1);
        build_graph(config1).unwrap();
        
        let config2 = BuildConfig::new(&repo, &graph2);
        build_graph(config2).unwrap();
        
        let graph1 = Graph::read_from_file(&graph1).unwrap();
        let graph2 = Graph::read_from_file(&graph2).unwrap();
        
        let diff = compare_graphs(&graph1, &graph2);
        assert!(diff.is_identical());
    }
}