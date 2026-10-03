use crate::graph::format::{Graph, GraphHeader, ArrayOffsets, GRAPH_MAGIC, GRAPH_VERSION};
use crate::git::{GitCommand, Invocation};
use crate::error::GitflowError;
use std::path::Path;
use std::collections::HashMap;

/// Configuration for graph builder
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub repo_path: std::path::PathBuf,
    pub output_path: std::path::PathBuf,
    pub include_all_refs: bool,
}

impl BuildConfig {
    pub fn new(repo_path: impl Into<std::path::PathBuf>, output_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            output_path: output_path.into(),
            include_all_refs: true,
        }
    }
}

/// Git log entry parsed from `git log --format`
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub oid: String,
    pub parents: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    pub author_date: i64,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: i64,
    pub subject: String,
    pub refs: Vec<String>,
    pub index: usize,
}

/// Build the commit graph from a repository
pub fn build_graph(config: BuildConfig) -> Result<(), GitflowError> {
    // 1. Run git log to get all commits
    let log_entries = run_git_log(&config.repo_path)?;
    
    // 2. Build OID -> index mapping
    let oid_to_index: HashMap<String, u32> = log_entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.oid.clone(), i as u32))
        .collect();
    
    // 3. Resolve parent OIDs to indices
    let mut parent_index = Vec::with_capacity(log_entries.len() + 1);
    let mut parent_list = Vec::new();
    let mut current_offset = 0u32;
    
    for entry in &log_entries {
        parent_index.push(current_offset);
        for parent_oid in &entry.parents {
            if let Some(&idx) = oid_to_index.get(parent_oid) {
                parent_list.push(idx);
            }
        }
        current_offset = parent_list.len() as u32;
    }
    parent_index.push(current_offset);
    
    // 4. Build commit data arrays
    let n = log_entries.len() as u32;
    let m = parent_list.len() as u32;
    
    let mut graph = super::format::Graph::new(n, m);
    graph.parent_index = parent_index;
    graph.parent_list = parent_list;
    
    // Fill commit data
    for (i, entry) in log_entries.iter().enumerate() {
        graph.commit_time[i] = entry.author_date;
        graph.lane[i] = 0; // Will be assigned by lane assignment
        graph.parent_count[i] = entry.parents.len().min(3) as u8;
        
        // Pack strings: subject\0author\0refs\0
        let refs_str = entry.refs.join(",");
        let string_block = format!(
            "{}\0{}\0{}\0",
            entry.subject, entry.author_name, entry.refs.join(",")
        );
        let start = graph.strings.len();
        graph.string_offsets[entry.index] = start as u32;
        graph.strings.extend(string_block.as_bytes());
        graph.strings.push(0); // null terminator
    }
    
    // 5. Assign lanes (topological lane assignment)
    assign_lanes(&mut graph);
    
    // 6. Write graph.bin
    graph.write_to_file(&config.output_path)?;
    
    Ok(())
}

/// Run git log and parse output
pub fn run_git_log(repo_path: &std::path::Path) -> Result<Vec<LogEntry>, GitflowError> {
    // Format: %H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%D
    // Separator: 0x1f (unit separator)
    let output = GitCommand::new(&repo_path, [
        "log",
        "--all",
        "--topo-order",
        "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%D",
    ]).run()?;
    
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_git_log(&stdout)
    } else {
        Ok(Vec::new())
    }
}

pub fn parse_git_log(output: &str) -> Result<Vec<LogEntry>, GitflowError> {
    let mut entries = Vec::new();
    
    for (index, line) in output.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split('\x1f').collect();
        if parts.len() < 10 {
            continue; // Skip malformed lines
        }
        
        let parents: Vec<String> = parts[1]
            .split(' ')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        let refs: Vec<String> = parts[9]
            .split(", ")
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        
        entries.push(LogEntry {
            oid: parts[0].to_string(),
            parents,
            author_name: parts[2].to_string(),
            author_email: parts[3].to_string(),
            author_date: parts[4].parse().unwrap_or(0),
            committer_name: parts[5].to_string(),
            committer_email: parts[6].to_string(),
            committer_date: parts[7].parse().unwrap_or(0),
            subject: parts[8].to_string(),
            refs,
            index,
        });
    }
    
    Ok(entries)
}

/// Lane assignment algorithm (from spike)
/// Assigns each commit to a lane for visualization
/// Commits in the same lane don't overlap horizontally
pub fn assign_lanes(graph: &mut super::format::Graph) {
    let n = graph.header.commit_count as usize;
    
    // Build adjacency list (children)
    let mut children: Vec<Vec<u32>> = vec![Vec::new(); n];
    for i in 0..n {
        let start = graph.parent_index[i] as usize;
        let end = graph.parent_index[i + 1] as usize;
        for &parent_idx in &graph.parent_list[start..end] {
            if (parent_idx as usize) < n {
                children[parent_idx as usize].push(i as u32);
            }
        }
    }
    
    // Lane assignment using free-lane pool
    let mut lanes = vec![-1i32; n];
    let mut free_lanes: Vec<i32> = (0..n as i32).collect();
    
    // Process in topo order (already in topo order from git log --topo-order)
    for i in 0..n {
        // Find lanes used by parents
        let mut used_lanes = std::collections::HashSet::new();
        let start = graph.parent_index[i] as usize;
        let end = graph.parent_index[i + 1] as usize;
        for &parent_idx in &graph.parent_list[start..end] {
            if (parent_idx as usize) < n {
                if lanes[parent_idx as usize] >= 0 {
                    used_lanes.insert(lanes[parent_idx as usize]);
                }
            }
        }
        
        // Find first free lane not used by parents
        let mut assigned_lane = 0;
        while used_lanes.contains(&assigned_lane) {
            assigned_lane += 1;
        }
        
        graph.lane[i] = assigned_lane;
    }
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
    fn test_build_linear_graph() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        init_test_repo(&repo);
        
        // Create 5 commits
        for i in 0..5 {
            fs::write(repo.join(format!("file{}.txt", i)), format!("content {}", i)).unwrap();
            Command::new("git").args(["add", "."]).current_dir(&repo).output().unwrap();
            Command::new("git").args(["commit", "-m", &format!("commit {}", i)]).current_dir(&repo).output().unwrap();
        }
        
        let output = dir.path().join("graph.bin");
        let config = BuildConfig::new(&repo, &output);
        build_graph(config).unwrap();
        
        // Verify graph was created
        let graph = crate::graph::format::Graph::read_from_file(&output).unwrap();
        assert_eq!(graph.header.magic, crate::graph::format::GRAPH_MAGIC);
        assert_eq!(graph.header.commit_count, 5);
        assert_eq!(graph.parent_index.len(), 6); // n+1
        
        // Verify CSR structure
        assert_eq!(graph.parent_index[0], 0);
        assert_eq!(graph.parent_index[5], 4); // 4 edges total
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