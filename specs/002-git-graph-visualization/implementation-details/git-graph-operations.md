# Git Graph Operations: Literal Commands & Observables

Feature 002 implementa el builder y renderer del grafo. Todas las operaciones Git son literales y medibles.

## Graph Builder Operations

| Operation | Command | Exit codes | Observables comparados |
| --- | --- | --- | --- |
| Extract commits | `git log --all --topo-order --format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s` | 0 success, 128 no commits | `graph.bin` bytes idnticos a spike |
| Refs | `git for-each-ref --format="%(refname) %(objectname)"` | 0 | Refs + OIDs en graph.bin |

## Graph Builder Algorithm (Port exacto de spike/build-graph.mjs)

```rust
// 1. Spawn git log con formato custom
// 2. Stream stdout, split por 0x1f
// 3. Fill oidByRow, subjectByRow, parentOidFlat
// 4. Resolve parent OIDs -> row indices (Uint32Array)
// 5. Assign lanes -> Int32Array con free-lane pool
// 6. Pack 5 typed arrays -> graph.bin (v1 header + CSR)

// Typed arrays (byte-identicos a spike):
// - n: u32 (commit count)
// - offset: Uint32Array (n+1) CSR offsets
// - parent: Uint32Array (edges)
// - time: Int64Array (author timestamp)
// - lane: Int32Array (assigned lane)
// - parentCount: Uint8Array (0..3+)
// - strings: pool null-terminated (subjects + authors + refs)
```

## Renderer Operations (TypeScript, port exacto spike/public/index.html)

| Operation | Implementation | Budget |
| --- | --- | --- |
| Decode graph.bin | `Uint8Array` -> typed arrays (zero-copy via `bytemuck`) | < 50ms |
| Virtualized viewport | Solo commits visibles en canvas | p50 <= 16.7ms |
| Pan/zoom | Transform matrix + invalidate rect | p99 <= 33.4ms |
| Lane rendering | Rectngulos por lane (Int32Array) | max <= 50ms |

## Tauri Command Surface

| Command | Handler | Payload |
| --- | --- | --- |
| `graph:load` | `invoke("graph:load", { repo: string })` | Returns `ArrayBuffer` (graph.bin) |
| `graph:build` | CLI `gitflowfy graph build <repo> <out>` | Stdout/stderr/exit |

## Differential Comparison (Feature 001 reuse)

| Observable | Command | Tolerance |
| --- | --- | --- |
| Graph OIDs | `git rev-list --all --topo-order` | Exact byte match |
| Graph structure | `graph.bin` bytes vs spike output | Byte-for-byte |
| Renderer frame time | `requestAnimationFrame` timing | p50 <= 16.7ms, p99 <= 33.4ms |

## Fixtures (Feature 001 reutilizadas)

| Fixture | Graph expectation |
| --- | --- |
| Linear | 1 lane, n commits, parent_count=1 |
| Merged | 2 lanes en merge commit, parent_count=2 |
| Octopus | 3+ lanes, parent_count=3+ |
| Orphan | 2 componentes desconectados |
| Detached | HEAD suelto, 1 commit visible |
| Empty | 0 commits, graph.bin vaco (n=0) |
| Path-edge | Paths con espacios/non-ASCII en refs |

## Error Handling

| Scenario | Behavior |
| --- | --- |
| Repo vaco | graph.bin con n=0, renderer muestra estado vaco |
| git log falla | Propaga exit code, stderr, sin graph.bin parcial |
| graph.bin corrupto | Renderer rechaza con error descriptivo |
| Tauri invoke timeout | 30s timeout, error propagado a UI |

## CI Gates (Article VI)

| Gate | Command | Threshold |
| --- | --- | --- |
| Extract | `cargo test --package gitflowfy-core --test perf_extract` | < 30s |
| Graph size | `cargo test --package gitflowfy-core --test graph_size` | < 32 MB |
| Load | `npm run bench:load` en crates/ui | < 2s |
| Frame p50 | `npm run bench:frame` | <= 16.7ms |
| Frame p99 | `npm run bench:frame` | <= 33.4ms |
| Max frame | `npm run bench:frame` | < 50ms |
| Cross-platform | CI Windows + Linux | Both pass |

---

*Implementation details version 1.0.0 - alineado con spec v1.0.0*