# Implementation Plan: Advanced Interactions

**Branch**: `005-advanced-interactions`
**Spec**: [spec.md](spec.md)
**Status**: Draft
**Created**: 2026-09-30

---

## Phase -1: Pre-Implementation Gates

### Constitution Check

| Article | Binding here? | How this plan satisfies it |
| --- | --- | --- |
| I - Shell out | **S** | Search/diff/blame usan `GitCommand` de Feature 001. No spawn directo. |
| II - Reflog only | **S** | Search index no mantiene estado repo. Solo cache LRU de queries. |
| III - Test first | **S** | Phase 0: search/diff tests antes que implementation. |
| IV - CLI first | **S** | `gitflowfy search`, `gitflowfy blame`, `gitflowfy diff`, `gitflowfy stash`, `gitflowfy worktree`, `gitflowfy remote` sin ventana. |
| V - Odd repo states | **S** | Search/diff/blame funcionan en estados Feature 003 (merge conflict, rebase, etc.) |
| VI - Perf budgets | **S** | NFR-001: search < 500ms, NFR-002: diff < 100ms, NFR-003: index < 100MB. |
| VII - <=3 crates | **S** | Logic en `crates/core` (search/diff/blame), UI en `crates/ui`. Total = 3/3. |
| VIII - Anti-abstraction | **S** | Myers O(ND) propio, LRU cache Map, `GitCommand` directo. Sin wrappers. |
| IX - Integration first | **S** | Tests contra `git` real + fixtures Feature 001/002/003. |

### Gates

| Gate | Pass Criteria |
| --- | --- |
| G1 | Feature 001 G1 passed |
| G2 | Feature 001 G2 passed (overhead budget) |
| G3 | Feature 001 G3 passed (fixtures) |
| G4 | No `git` spawn fuera de `crates/core` |
| G5 | No `libgit2`/`gix`/`jgit` |

---

## Technical Approach

### Stack Decision

**Elegido**:
- **Search/Diff/Blame core**: Rust en `crates/core/src/search/`, `crates/core/src/diff/`, `crates/core/src/blame/`
- **Search Index**: LRU cache en memoria (Map/Trie) - 100 entries max
- **Diff Algorithm**: Myers O(ND) en TypeScript (portado a Rust para core)
- **Diff Viewer**: DOM virtualizado + Canvas minimap (hybrid)
- **Search Index Storage**: LRU cache en memoria (Map/Trie), 100 entries
- **Search Scope**: Repo actual + toggle "All Repos"
- **UI**: SolidJS en `crates/ui` (reusa stack Feature 004)

### Architecture

```
crates/core/
  src/
    search/
      mod.rs
      index.rs          # LRU cache + git log/grep on-demand
      query.rs          # SearchQuery, SearchResult, filters
    diff/
      mod.rs
      myers.rs          # Myers O(ND) algorithm (port TS -> Rust)
      format.rs         # Diff formatting (side-by-side, inline, word-level)
    blame/
      mod.rs
      annotate.rs       # git blame parsing -> line annotations
    cli/
      mod.rs            # search, blame, diff, stash, worktree, remote subcommands

crates/ui/
  src/
    search/
      SearchPanel.tsx       # Search bar + filters + results
      SearchIndex.ts        # LRU cache client-side
    diff/
      DiffViewer.tsx        # DOM virtualizado + Canvas minimap
      DiffFormat.tsx        # Side-by-side, inline, word-level
    blame/
      BlamePanel.tsx        # Line annotations + heatmap
      HistoryView.tsx       # File history timeline
    stash/
      StashList.tsx         # List, apply, pop, drop, rename, branch
    worktree/
      WorktreeList.tsx      # List, add, remove, move, prune
    remote/
      RemoteManager.tsx     # Add, remove, edit, fetch, prune
    components/
      SearchBar.tsx         # Global search bar + scope toggle
      DiffViewer.tsx        # Reusable
      BlameView.tsx         # Reusable

apps/desktop/
  # Existente
```

### Data Flow

1. **Search**: User types -> `SearchBar` -> `invoke("search:query", { repo, query, filters })` -> `crates/core/search` -> `git log/grep` + LRU cache -> results -> render
2. **Blame**: File open -> `invoke("blame:annotate", { repo, file })` -> `crates/core/blame` -> `git blame -L` -> line annotations -> render
3. **Diff**: Commit selected -> `invoke("diff:compare", { repo, base, target })` -> `crates/core/diff` -> Myers O(ND) -> `DiffResult` -> virtualized DOM + Canvas minimap
4. **Stash/Worktree/Remote**: UI -> `invoke("stash:list", ...)` / `invoke("worktree:list", ...)` / `invoke("remote:list", ...)` -> `GitCommand` -> results

### Search Index (LRU Cache)

```rust
// crates/core/src/search/index.rs
struct SearchIndex {
    cache: LruCache<String, SearchResults>, // key = query+filters hash
    max_entries: 100,
}

impl SearchIndex {
    fn search(&mut self, repo: &Path, query: &SearchQuery) -> SearchResults {
        let key = hash(query);
        if let Some(cached) = self.cache.get(&key) {
            return cached;
        }
        let results = self.run_git_search(repo, query)?;
        self.cache.put(key, results.clone());
        results
    }

    fn run_git_search(&self, repo: &Path, query: &SearchQuery) -> SearchResults {
        // git log --all --grep="query" --author="x" --since="y" --until="z" -- <paths>
        // git grep -n "pattern" -- <paths>
    }
}
```

### Myers O(ND) Diff (TypeScript -> Rust)

```typescript
// Simplified Myers O(ND) for diff lines
function myersDiff(oldLines: string[], newLines: string[]): EditScript {
    // O((N+M)D) where D = edit distance
    // Returns EditScript: [{ type: 'equal'|'delete'|'insert', value: string }]
}
```

**Port to Rust**: Same algorithm in `crates/core/src/diff/myers.rs`. Used by both CLI and webview (via WASM compile or native).

### Diff Viewer (Hybrid DOM + Canvas)

```typescript
// DiffViewer.tsx
const DiffViewer = ({ diff }: { diff: EditScript }) => {
  const virtualList = useVirtualList(diff, { itemHeight: 20, overscan: 10 });
  const minimapRef = useRef<HTMLCanvasElement>(null);
  
  return (
    <div className="diff-viewer">
      <canvas ref={minimapRef} width={200} height={400} className="minimap" />
      <VirtualList
        items={virtualList}
        renderItem={({ item, index }) => (
          <DiffLine 
            type={item.type} 
            content={item.value} 
            lineNumber={index}
            wordDiff={item.wordDiff}
          />
        )}
      />
    </div>
  );
};
```

### Tauri Invoke Surface

| Command | Payload | Returns |
| --- | --- | --- |
| `search:query` | `{ repoId, query, filters, scope }` | `SearchResults` |
| `blame:annotate` | `{ repoId, file, lines? }` | `BlameAnnotations` |
| `diff:compare` | `{ repoId, base, target, format }` | `DiffResult` |
| `stash:list` | `{ repoId }` | `StashEntry[]` |
| `stash:action` | `{ repoId, index, action }` | `Invocation` |
| `worktree:list` | `{ repoId }` | `WorktreeEntry[]` |
| `worktree:action` | `{ repoId, action, args }` | `Invocation` |
| `remote:list` | `{ repoId }` | `RemoteEntry[]` |
| `remote:action` | `{ repoId, action, args }` | `Invocation` |
| `commit:actions` | `{ repoId, action, range }` | `Invocation` |
| `tag:action` | `{ repoId, action, name, msg }` | `Invocation` |

---

## Requirement Traceability

| Requirement | Technical decision | Verified by |
| --- | --- | --- |
| Story 1: Global Search | `search/index.rs` LRU + `git log/grep` | `t_search_perf`, `t_search_filters` |
| Story 2: File Search | `search/index.rs` + `git grep` | `t_file_search_perf`, `t_file_search_grep` |
| Story 3: Blame/History | `blame/annotate.rs` + `git blame` | `t_blame_accuracy`, `t_blame_heatmap` |
| Story 4: Multi-Repo | `crates/ui/src/dashboard/` + Feature 001 | `t_dashboard_multi_repo` |
| Story 5: Diff Viewer | `diff/myers.rs` + `DiffViewer.tsx` | `t_diff_accuracy`, `t_diff_perf` |
| Story 6: Stash | `stash/` + `GitCommand` | `t_stash_list`, `t_stash_actions` |
| Story 7: Worktree | `worktree/` + `GitCommand` | `t_worktree_crud` |
| Story 8: Commit Actions | `cli/commit_actions.rs` | `t_cherry_pick_range`, `t_revert_range`, `t_bisect_ui` |
| Story 9: Remote | `remote/` + `GitCommand` | `t_remote_crud`, `t_fetch_prune` |
| NFR-001: Search < 500ms | LRU cache + on-demand | `t_search_perf_500ms` |
| NFR-002: Diff < 100ms | Myers O(ND) + virtualized DOM | `t_diff_perf_100ms` |
| NFR-003: Index < 100MB | LRU 100 entries | `t_index_size` |
| NFR-004: Cross-platform | Rust + TS + Tauri | CI Windows + Linux |
| NFR-005: a11y | Semantic HTML + ARIA | `axe-core` CI |

---

## Performance Impact

| Metric | Budget | New expected | Measured |
| --- | --- | --- | --- |
| Search (cache hit) | < 50 ms | < 10 ms | `t_search_cache_hit` |
| Search (cache miss) | < 500 ms | < 300 ms | `t_search_cache_miss` |
| Diff render (10k lines) | < 100 ms | < 50 ms | `t_diff_render` |
| Blame annotate (10k lines) | < 200 ms | < 150 ms | `t_blame_perf` |
| Search index size | < 100 MB | < 10 MB (100 entries LRU) | `t_index_size` |

---

## Project Structure

```
crates/core/
  src/
    search/
      mod.rs
      index.rs          # LRU cache
      query.rs
    diff/
      mod.rs
      myers.rs          # Myers O(ND)
      format.rs
    blame/
      mod.rs
      annotate.rs
    cli/
      mod.rs
      search.rs
      diff.rs
      blame.rs
      stash.rs
      worktree.rs
      remote.rs
      commit_actions.rs
      tag.rs

crates/ui/
  src/
    search/
      SearchPanel.tsx
      SearchIndex.ts
    diff/
      DiffViewer.tsx
      DiffFormat.tsx
    blame/
      BlamePanel.tsx
      HistoryView.tsx
    stash/
      StashList.tsx
    worktree/
      WorktreeList.tsx
    remote/
      RemoteManager.tsx
    components/
      SearchBar.tsx

apps/desktop/
  # Existente
```

---

## Complexity Tracking

| Exception | Article | Justification | Revisit when |
| --- | --- | --- | --- |
| Myers O(ND) propio | VIII | Control total word-level/inline/side-by-side. 0 deps. | Si algoritmo buggy. |
| LRU cache Map/Trie | VIII | Simple, 0 deps. No wrapper. | Si memoria > 100MB. |
| Virtual list DOM | VIII | Performance + accessibility nativa. | Si performance insuficiente. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| R1: Myers O(ND) bug en edge cases | Media | Diff incorrecto | Tests exhaustivos vs `git diff` output. Property-based testing. |
| R2: Search index memory leak | Baja | Memoria crece | LRU eviction estricto + tests de memoria. |
| R3: Virtual list performance 100k+ lines | Media | Frame drop | Virtualizacin + overscan + memoizacin. Profile en CI. |
| R4: Blame performance archivo grande | Media | UI freeze | WebWorker para blame parse, streaming results. |

---

## Plan Self-Review

- [x] Every requirement traced to decision + test
- [x] All 5 clarifications resolved in spec
- [x] Every git command spelled out literally
- [x] Headless surface defined before GUI (CLI invoke before UI)
- [x] Fixtures for all in-scope states (reused from 001/002/003)
- [x] Perf budget stated + measurement (NFR-001..005)
- [x] Complexity exceptions justified (3, each with trigger)
- [x] Pure ASCII output

---

*Plan version 1.0.0 - listo para revisin constitucional*