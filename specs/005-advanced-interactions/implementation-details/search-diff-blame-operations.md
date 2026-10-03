# Advanced Interactions: Literal Commands & Observables

Feature 005 implementa bsqueda, diff, blame, stash, worktree, remote, commit actions avanzados.

## Search Operations

| Operation | Command | Filters | Observables |
| --- | --- | --- | --- |
| Global commit search | `git log --all --grep="<query>" --author="<x>" --since="<y>" --until="<z>" -- <paths>` | query, author, date range, paths | Commit list: hash, subject, author, date |
| File search (name) | `git ls-files -- <pattern>` | glob pattern, extension | File list: path, mode, stage |
| File content search | `git grep -n -i -- <pattern> -- <paths>` | pattern, case-insensitive, paths | Matches: file, line, column, context |
| Multi-repo search | Parallel `git log/grep` per repo | repo scope toggle | Aggregated results per repo |

## Diff Operations

| Operation | Command | Formats | Observables |
| --- | --- | --- | --- |
| Commit diff | `git diff <base> <target> -- <paths>` | side-by-side, inline, word-level | EditScript: equal/delete/insert + word-diff |
| Range diff | `git diff <A>..<B> -- <paths>` | same | Same |
| File diff | `git diff <base> <target> -- <file>` | same | Same |
| Whitespace ignore | `git diff -w -- <paths>` | same | Same |

## Blame Operations

| Operation | Command | Output |
| --- | --- | --- |
| File blame | `git blame -L <start>,<end> -- <file>` | Line annotations: author, date, commit, line |
| File history | `git log --oneline -- <file>` | Commit list per file |
| Heatmap data | `git log --format=%at -- <file>` | Timestamps per line for heatmap |

## Stash Operations

| Operation | Command | Exit Codes |
| --- | --- | --- |
| List stashes | `git stash list --format="%gd %gs %cr"` | 0 |
| Apply stash | `git stash apply <stash>` | 0 success, 1 conflict |
| Pop stash | `git stash pop <stash>` | 0 success, 1 conflict |
| Drop stash | `git stash drop <stash>` | 0 success |
| Rename stash | `git stash rename <n> <new-name>` | 0 success |
| Branch from stash | `git stash branch <name> <stash>` | 0 success |

## Worktree Operations

| Operation | Command | Exit Codes |
| --- | --- | --- |
| List worktrees | `git worktree list --porcelain` | 0 |
| Add worktree | `git worktree add <path> <branch>` | 0 success |
| Remove worktree | `git worktree remove <path>` | 0 success |
| Move worktree | `git worktree move <old> <new>` | 0 success |
| Prune worktrees | `git worktree prune` | 0 success |

## Remote Operations

| Operation | Command | Exit Codes |
| --- | --- | --- |
| List remotes | `git remote -v` | 0 |
| Add remote | `git remote add <name> <url>` | 0 success |
| Remove remote | `git remote remove <name>` | 0 success |
| Rename remote | `git remote rename <old> <new>` | 0 success |
| Set URL | `git remote set-url <name> <url>` | 0 success |
| Fetch all | `git fetch --all --prune` | 0 success |
| Push tags | `git push --tags` | 0 success |

## Commit Actions (Advanced)

| Operation | Command | Exit Codes |
| --- | --- | --- |
| Cherry-pick range | `git cherry-pick <A>..<B>` | 0 success, 1 conflict |
| Revert range | `git revert <A>..<B>` | 0 success, 1 conflict |
| Bisect start | `git bisect start <bad> <good>` | 0 |
| Bisect good | `git bisect good <rev>` | 0 |
| Bisect bad | `git bisect bad <rev>` | 0 |
| Bisect skip | `git bisect skip <rev>` | 0 |
| Bisect reset | `git bisect reset` | 0 |
| Tag create | `git tag -a <tag> -m <msg> <rev>` | 0 success |
| Tag delete | `git tag -d <tag>` | 0 success |
| Tag push | `git push --tags` | 0 success |

## Search Index (LRU Cache)

```rust
// crates/core/src/search/index.rs
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

struct SearchIndex {
    cache: LruCache<u64, SearchResults>,
    max_entries: usize,
}

impl SearchIndex {
    fn new(max_entries: usize) -> Self {
        Self { cache: LruCache::new(max_entries), max_entries }
    }

    fn make_key(query: &SearchQuery) -> u64 {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    fn search(&mut self, repo: &Path, query: SearchQuery) -> SearchResults {
        let key = Self::make_key(&query);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        let results = Self::run_git_search(&repo, &query)?;
        self.cache.put(Self::make_key(&query), results.clone());
        results
    }

    fn run_git_search(repo: &Path, query: &SearchQuery) -> SearchResults {
        // Build git log/grep command from query
        // Execute via GitCommand
        // Parse output to SearchResults
    }
}
```

## Myers O(ND) Diff Algorithm (TypeScript -> Rust)

```typescript
// Simplified Myers O(ND) for line diff
interface Edit { type: 'equal' | 'delete' | 'insert'; value: string; wordDiff?: WordDiff[]; }

function myersDiff(oldLines: string[], newLines: string[]): Edit[] {
    const N = oldLines.length, M = newLines.length;
    const maxD = N + M;
    const V = new Array(2 * maxD + 1).fill(-1);
    V[1] = 0;
    
    for (let D = 0; D <= maxD; D++) {
        for (let k = -D; k <= D; k += 2) {
            let x = (k === -D || (k !== D && V[maxD + k - 1] < V[maxD + k + 1])) 
                ? V[maxD + k + 1] 
                : V[maxD + k - 1] + 1;
            let y = x - k;
            
            while (x < N && y < M && oldLines[x] === newLines[y]) {
                x++; y++;
            }
            V[maxD + k] = x;
            
            if (x >= N && y >= M) {
                // Trace back to build edit script
                return traceBack(V, oldLines, newLines, D);
            }
        }
    }
    return []; // Should not reach
}

function traceBack(V: number[], oldLines: string[], newLines: string[], D: number): Edit[] {
    // Trace back through V to build edit script
    // Returns array of { type: 'equal'|'delete'|'insert', value: string }
}
```

**Rust port**: Mismo algoritmo en `crates/core/src/diff/myers.rs`. Compilado a WASM para webview o nativo.

## Diff Viewer (Hybrid DOM + Canvas)

```typescript
// DiffViewer.tsx
const DiffViewer = ({ editScript }: { editScript: Edit[] }) => {
  const virtualList = useVirtualList(editScript, { 
    itemHeight: 20, 
    overscan: 10,
    estimateSize: () => 20 
  });
  
  const minimapRef = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const canvas = minimapRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d')!;
    // Draw minimap: red=delete, green=insert, gray=equal
  }, [editScript]);
  
  return (
    <div className="diff-viewer" style={{ display: 'flex' }}>
      <canvas ref={minimapRef} width={200} height="100%" className="minimap" />
      <VirtualList
        items={virtualList}
        renderItem={({ item, index }) => (
          <DiffLine 
            type={item.type} 
            content={item.value} 
            lineNumber={index}
            wordDiff={item.wordDiff}
            onSelect={handleLineSelect}
          />
        )}
      />
    </div>
  );
};

// DiffLine.tsx
const DiffLine = ({ type, content, lineNumber, wordDiff }) => {
  const bgColor = type === 'insert' ? '#e6ffed' : 
                  type === 'delete' ? '#ffeef0' : 'transparent';
  
  return (
    <div className="diff-line" style={{ backgroundColor: bgColor }}>
      <span className="line-num">{lineNumber}</span>
      <span className={`diff-type-${type}`}>
        {type === 'insert' ? '+' : type === 'delete' ? '-' : ' '}
      </span>
      <WordDiff content={content} wordDiff={wordDiff} />
    </div>
  );
};
```

## Search LRU Cache (Client + Server)

```typescript
// SearchIndex.ts (client-side cache mirror)
class SearchIndex {
  private cache = new Map<string, { results: SearchResults; timestamp: number }>();
  private maxEntries = 100;
  
  async search(query: SearchQuery): Promise<SearchResults> {
    const key = this.hashQuery(query);
    const cached = this.cache.get(key);
    if (cached && Date.now() - cached.timestamp < 5 * 60 * 1000) { // 5 min TTL
      return cached.results;
    }
    const results = await invoke('search:query', { query });
    this.cache.set(key, { results, timestamp: Date.now() });
    if (this.cache.size > this.maxEntries) {
      const oldest = this.cache.keys().next().value;
      this.cache.delete(oldest);
    }
    return results;
  }
  
  private hashQuery(query: SearchQuery): string {
    return btoa(JSON.stringify(query)).slice(0, 32);
  }
}
```

## Tests

| Test | Description |
| --- | --- |
| `t_search_global` | Global commit search con filtros (autor, fecha, paths) |
| `t_search_file_name` | File search por nombre glob pattern |
| `t_search_file_content` | `git grep` content search con context |
| `t_search_multi_repo` | Scope toggle "All Repos" |
| `t_search_perf` | < 500ms cache miss, < 50ms cache hit |
| `t_diff_myers` | Myers O(ND) vs `git diff` output exacto |
| `t_diff_formats` | Side-by-side, inline, word-level, ignore whitespace |
| `t_diff_perf` | 10k lines < 100ms render |
| `t_blame_annotate` | `git blame` output parse exacto |
| `t_blame_heatmap` | Heatmap timestamps por lnea |
| `t_stash_crud` | List, apply, pop, drop, rename, branch |
| `t_worktree_crud` | List, add, remove, move, prune |
| `t_remote_crud` | List, add, remove, edit, fetch, prune, push tags |
| `t_commit_actions` | Cherry-pick range, revert range, bisect, tag |
| `t_search_perf` | < 500ms cold, < 50ms cache hit |
| `t_diff_perf` | 10k lines < 100ms render |
| `t_index_size` | LRU 100 entries < 10MB |

---

*Implementation details version 1.0.0 - alineado con spec v1.0.0*