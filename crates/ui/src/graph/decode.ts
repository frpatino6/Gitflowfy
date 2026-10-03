/**
 * Decode graph.bin binary format into typed arrays for rendering
 * Matches the exact binary format from spike/build-graph.mjs
 */

export interface GraphData {
  n: number;           // number of commits
  m: number;           // number of edges
  lanes: number;       // number of lanes
  parentIndex: Uint32Array;  // CSR offsets (n+1)
  parentList: Uint32Array;   // CSR edges (m)
  commitTime: BigInt64Array; // author timestamp
  lane: Int32Array;          // assigned lane per commit
  parentCount: Uint8Array;   // 0,1,2,3+ (3 = 3+ parents)
  subjects: string[];        // commit subjects
  authors: string[];         // author names
  refs: string[];            // ref names per commit
}

const GRAPH_MAGIC = 0x47524150; // "GRAP"
const GRAPH_VERSION = 1;
const HEADER_SIZE = 16; // 4+1+3+4+4 = 16

/**
 * Decode graph.bin binary format into typed arrays for rendering
 * Binary format (v1):
 * - Header: 16 bytes (magic:u32, version:u8, padding:3, n:u32, m:u32)
 * - parent_index: (n+1) * u32 (CSR offsets)
 * - parent_list: m * u32 (CSR edges)
 * - commit_time: n * i64 (author timestamps)
 * - lane: n * i32 (assigned lane)
 * - parent_count: n * u8 (0-3)
 * - strings: variable length, packed as subject\0author\0refs\0 per commit
 */
export function decodeGraph(data: Uint8Array): GraphData {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 0;
  
  const magic = view.getUint32(0, true);
  if (magic !== GRAPH_MAGIC) {
    throw new Error(`Invalid graph magic: 0x${magic.toString(16)}`);
  }
  
  const version = view.getUint8(4);
  if (version !== GRAPH_VERSION) {
    throw new Error(`Unsupported graph version: ${version}`);
  }
  
  const n = view.getUint32(8, true);
  const m = view.getUint32(12, true);
  
  // Calculate offsets
  const parentIndexOffset = HEADER_SIZE;
  const parentListOffset = HEADER_SIZE + (n + 1) * 4;
  const commitTimeOffset = HEADER_SIZE + (n + 1) * 4 + m * 4;
  const laneOffset = commitTimeOffset + n * 8;
  const parentCountOffset = laneOffset + n * 4;
  const stringsOffset = parentCountOffset + n;
  
  // Parse typed arrays
  const parentIndex = new Uint32Array(data.buffer, data.byteOffset + parentIndexOffset, n + 1);
  const parentList = new Uint32Array(data.buffer, data.byteOffset + parentListOffset, m);
  const commitTime = new BigInt64Array(data.buffer, data.byteOffset + commitTimeOffset, n);
  const lane = new Int32Array(data.buffer, data.byteOffset + laneOffset, n);
  const parentCount = new Uint8Array(data.buffer, data.byteOffset + parentCountOffset, n);
  
  // Parse string pool
  const stringsData = new Uint8Array(data.buffer, data.byteOffset + stringsOffset, data.byteLength - stringsOffset);
  const strings = new TextDecoder().decode(stringsData);
  
  // Parse string pool: subject\0author\0refs\0 per commit
  const subjects: string[] = [];
  const authors: string[] = [];
  const refs: string[] = [];
  
  let rest = strings;
  for (let i = 0; i < n; i++) {
    const subjectEnd = rest.indexOf('\0');
    if (subjectEnd === -1) break;
    const subject = rest.substring(0, subjectEnd);
    rest = rest.slice(subjectEnd + 1);
    subjects.push(subject);
    
    const authorEnd = rest.indexOf('\0');
    if (authorEnd === -1) break;
    const author = rest.substring(0, authorEnd);
    rest = rest.slice(authorEnd + 1);
    authors.push(author);
    
    const refEnd = rest.indexOf('\0');
    if (refEnd === -1) break;
    const ref = rest.substring(0, refEnd);
    rest = rest.slice(refEnd + 1);
    refs.push(ref);
  }
  
  // Calculate max lane for layout
  let maxLane = 0;
  for (let i = 0; i < n; i++) {
    if (lane[i] > maxLane) maxLane = lane[i];
  }
  
  return {
    n,
    m,
    lanes: maxLane + 1,
    parentIndex,
    parentList,
    commitTime,
    lane,
    parentCount,
    subjects,
    authors,
    refs,
  };
}