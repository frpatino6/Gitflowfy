use bytemuck::{Pod, Zeroable};
use std::io::{Read, Write};
use std::path::Path;

/// Magic number for graph.bin: "GRAP" (0x47524150)
pub const GRAPH_MAGIC: u32 = 0x47524150;
pub const GRAPH_VERSION: u8 = 1;

/// Header of graph.bin (v1)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GraphHeader {
    pub magic: u32,
    pub version: u8,
    pub _padding: [u8; 3],
    pub commit_count: u32,
    pub edge_count: u32,
}

/// Offsets in the binary file
pub const HEADER_SIZE: usize = std::mem::size_of::<GraphHeader>();

/// Offsets for each array (after header)
pub fn array_offsets(commit_count: u32, edge_count: u32) -> ArrayOffsets {
    let mut offset = HEADER_SIZE as u64;
    
    let offset = ArrayOffsets {
        parent_index: offset,
        parent_list: offset + (commit_count as u64 + 1) * 4,
        commit_time: offset + (commit_count as u64 + 1) * 4 + (edge_count as u64) * 4,
        lane: offset + (commit_count as u64 + 1) * 4 + (edge_count as u64) * 4 + (commit_count as u64) * 8,
        parent_count: offset + (commit_count as u64 + 1) * 4 + (edge_count as u64) * 4 + (commit_count as u64) * 8 + (commit_count as u64) * 4,
    };
    offset
}

#[derive(Debug, Clone, Copy)]
pub struct ArrayOffsets {
    pub parent_index: u64,
    pub parent_list: u64,
    pub commit_time: u64,
    pub lane: u64,
    pub parent_count: u64,
}

/// Graph data structure (in-memory representation)
#[derive(Debug, Clone)]
pub struct Graph {
    pub header: GraphHeader,
    pub parent_index: Vec<u32>,    // CSR offsets: len = n+1
    pub parent_list: Vec<u32>,     // CSR edges: len = m
    pub commit_time: Vec<i64>,     // author timestamp
    pub lane: Vec<i32>,            // assigned lane
    pub parent_count: Vec<u8>,     // 0, 1, 2, 3+ (3 = 3 or more)
    pub strings: Vec<u8>,          // null-terminated pool: subjects + authors + refs
    pub string_offsets: Vec<u32>,  // offsets into strings pool
}

impl Graph {
    pub fn new(commit_count: u32, edge_count: u32) -> Self {
        Self {
            header: GraphHeader {
                magic: GRAPH_MAGIC,
                version: GRAPH_VERSION,
                _padding: [0, 0, 0],
                commit_count,
                edge_count,
            },
            parent_index: vec![0; (commit_count + 1) as usize],
            parent_list: Vec::with_capacity(edge_count as usize),
            commit_time: vec![0; commit_count as usize],
            lane: vec![0; commit_count as usize],
            parent_count: vec![0; commit_count as usize],
            strings: Vec::new(),
            string_offsets: vec![0; commit_count as usize],
        }
    }

    /// Total size in bytes
    pub fn size_bytes(&self) -> u64 {
        HEADER_SIZE as u64
            + (self.parent_index.len() * 4) as u64
            + (self.parent_list.len() * 4) as u64
            + (self.commit_time.len() * 8) as u64
            + (self.lane.len() * 4) as u64
            + (self.parent_count.len() * 1) as u64
            + self.strings.len() as u64
    }

    /// Write to file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        self.write_to_writer(&mut std::io::BufWriter::new(file))
    }

    /// Write to any writer
    pub fn write_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Write header
        let header_bytes = bytemuck::bytes_of(&self.header);
        writer.write_all(header_bytes)?;
        
        // Write parent_index
        let parent_index_bytes = bytemuck::cast_slice(&self.parent_index);
        writer.write_all(parent_index_bytes)?;
        
        // Write parent_list
        let parent_list_bytes = bytemuck::cast_slice(&self.parent_list);
        writer.write_all(parent_list_bytes)?;
        
        // Write commit_time
        let commit_time_bytes = bytemuck::cast_slice(&self.commit_time);
        writer.write_all(commit_time_bytes)?;
        
        // Write lane
        let lane_bytes = bytemuck::cast_slice(&self.lane);
        writer.write_all(lane_bytes)?;
        
        // Write parent_count
        writer.write_all(&self.parent_count)?;
        
        // Write strings
        writer.write_all(&self.strings)?;
        
        Ok(())
    }

    /// Write to a byte vector
    pub fn write_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.size_bytes() as usize);
        self.write_to_writer(&mut buf)?;
        Ok(buf)
    }

    /// Read from file
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        if buffer.len() < HEADER_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File too small for header",
            ));
        }
        
        // Read header
        let header = bytemuck::from_bytes::<GraphHeader>(&buffer[..HEADER_SIZE]);
        
        if header.magic != GRAPH_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid magic: 0x{:08x}", header.magic),
            ));
        }
        
        if header.version != GRAPH_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unsupported version: {}", header.version),
            ));
        }
        
        let n = header.commit_count as usize;
        let m = header.edge_count as usize;
        
        let offsets = array_offsets(header.commit_count, header.edge_count);
        
        // Parse arrays
        let parent_index = bytemuck::cast_slice::<u8, u32>(
            &buffer[offsets.parent_index as usize..offsets.parent_list as usize]
        ).to_vec();
        
        let parent_list = bytemuck::cast_slice::<u8, u32>(
            &buffer[offsets.parent_list as usize..offsets.commit_time as usize]
        ).to_vec();
        
        let commit_time = bytemuck::cast_slice::<u8, i64>(
            &buffer[offsets.commit_time as usize..offsets.lane as usize]
        ).to_vec();
        
        let lane = bytemuck::cast_slice::<u8, i32>(
            &buffer[offsets.lane as usize..offsets.parent_count as usize]
        ).to_vec();
        
        let parent_count = buffer[offsets.parent_count as usize..offsets.parent_count as usize + n]
            .to_vec();
        
        let strings = buffer[offsets.parent_count as usize + n..].to_vec();
        
        // Build string_offsets (not stored in file, computed on load)
        let mut string_offsets = vec![0; n];
        let mut pos = 0;
        for i in 0..n {
            string_offsets[i] = pos as u32;
            // Find next null terminator
            while pos < strings.len() && strings[pos] != 0 {
                pos += 1;
            }
            pos += 1; // skip null
        }
        
        Ok(Self {
            header: *header,
            parent_index,
            parent_list,
            commit_time,
            lane,
            parent_count,
            strings,
            string_offsets,
        })
    }

    /// Get subject string for commit
    pub fn subject(&self, index: usize) -> Option<&str> {
        if index >= self.string_offsets.len() {
            return None;
        }
        let start = self.string_offsets[index] as usize;
        let end = if index + 1 < self.string_offsets.len() {
            self.string_offsets[index + 1] as usize
        } else {
            self.strings.len()
        };
        if start >= self.strings.len() {
            return None;
        }
        let slice = &self.strings[start..end.min(self.strings.len())];
        std::str::from_utf8(slice).ok()
    }

    /// Get author name for commit
    pub fn author(&self, index: usize) -> Option<&str> {
        // Author is stored after subject in string pool
        // Implementation depends on string packing order
        self.subject(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("graph.bin");
        
        let mut graph = Graph::new(3, 2);
        graph.parent_index = vec![0, 1, 2, 2];
        graph.parent_list = vec![0, 1];
        graph.commit_time = vec![1000, 2000, 3000];
        graph.lane = vec![0, 1, 0];
        graph.parent_count = vec![1, 1, 0];
        
        // Pack strings: subject\0author\0ref\0 for each commit
        let strings = [
            "initial commit\0Gitflowfy Test\0refs/heads/main\0",
            "second commit\0Gitflowfy Test\0refs/heads/main\0",
            "third commit\0Gitflowfy Test\0refs/heads/feature\0",
        ].join("\0") + "\0";
        graph.strings = strings.into_bytes();
        
        graph.write_to_file(&path).unwrap();
        let loaded = Graph::read_from_file(&path).unwrap();
        
        assert_eq!(loaded.header.magic, GRAPH_MAGIC);
        assert_eq!(loaded.header.version, GRAPH_VERSION);
        assert_eq!(loaded.header.commit_count, 3);
        assert_eq!(loaded.header.edge_count, 2);
        assert_eq!(loaded.parent_index, vec![0, 1, 2, 2]);
        assert_eq!(loaded.parent_list, vec![0, 1]);
        assert_eq!(loaded.commit_time, vec![1000, 2000, 3000]);
        assert_eq!(loaded.lane, vec![0, 1, 0]);
        assert_eq!(loaded.parent_count, vec![1, 1, 0]);
    }
}