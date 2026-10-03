# Research: Advanced Interactions

La evidencia detrs de las decisiones en `plan.md`.

## Pregunta

Cmo implementar bsqueda, diff, blame, stash, worktree, remote avanzados sin violar Artculos I-IX.

## Lo que ya existe

| Feature | Qu entrega |
| --- | --- |
| 001 Headless | `GitCommand`, `AuditLog`, `RepoLock`, `GitCommand` (nico spawn site) |
| 002 Graph | `graph.bin` CSR, Canvas renderer, Tauri webview |
| 003 State | `RepoState` detection (10 estados), `RecoveryAction`, graph markers |
| 004 Chrome | SolidJS UI, Tauri v2, toolbars, mens, sidebars, shortcuts |

## Decisiones clave

### 1. Search: On-demand + LRU cache (no watchman)

| Opcin | Pros | Contras |
| --- | --- | --- |
| Watchman incremental | Bsquedas instantneas | Depende de watchman (no en Windows CI), complejidad |
| On-demand puro | Simple, 0 deps | Lento en repeat searches |
| **Hybrid: on-demand + LRU** | Primer search paga, repeats instantneos | Cache invalidation simple |

**Decisin**: LRU cache 100 entries en memoria (Map/Trie). Key = hash(query+filters). TTL 5 min. Primer search paga `git log/grep`, repeats leen cache. 100 entries = ~10MB.

### 2. Diff: Myers O(ND) propio (no diff-match-patch)

| Opcin | Pros | Contras |
| --- | --- | --- |
| diff-match-patch | Probado, ~10KB | Menos control word-level/inline |
| `git diff` + parse | Delegar a Git | Parse overhead, menos control |
| **Myers O(ND) propio** | Control total word-level/inline/side-by-side | ~200 lneas port |

**Decisin**: Myers O(ND) propio en TypeScript (portado a Rust). Control total para word-level diff, inline, side-by-side, ignore whitespace. 0 deps extra. Artculo VIII.

### 3. Diff Viewer: DOM virtualizado + Canvas minimap

| Opcin | Pros | Contras |
| --- | --- | --- |
| Canvas puro | Performance | Seleccin/copy/accessibility complejos |
| DOM virtualizado | Nativo selection/copy/accessibility | Virtualizacin = performance |
| **Hybrid: DOM virtualizado + Canvas minimap** | Best of both | Ligeramente ms cdigo |

**Decisin**: DOM virtualizado (virtual list) para diff lines + Canvas minimap para overview. Virtual list = nativo selection/copy/accessibility. Canvas minimap = overview rpido.

### 4. Search Index: LRU en memoria (no SQLite/IndexedDB)

| Opcin | Pros | Contras |
| --- | --- | --- |
| SQLite (sql.js WASM) | Persistente, query SQL | WASM overhead, ~500KB |
| IndexedDB | Persistente, nativo browser | Async, complejo |
| **LRU Map/Trie en memoria** | Simple, 0 deps, rpido | Se pierde al cerrar |

**Decisin**: LRU cache en memoria (Map/Trie), 100 entries max. Sin persistencia. Si usuario quiere persistir, feature posterior. 100 entries = ~10MB.

### 5. Diff Algorithm: Myers O(ND) portado

El algoritmo Myers O(ND) es el estndar para diff. Complejidad O((N+M)D) donde D = edit distance. Para diff de lneas tpicas (D << N+M), muy rpido.

Puerto TypeScript -> Rust para core. Compilado a WASM para webview o nativo.

### 6. Blame: `git blame` parse directo

`git blame -L <start>,<end> -- <file>` -> parse output -> line annotations. Heatmap: `git log --format=%at -- <file>` para timestamps por lnea.

### 5. Stash/Worktree/Remote: `GitCommand` directo

Todas las operaciones usan `GitCommand` de Feature 001:
- Single spawn site
- Audit log automtico
- Refusal network ops
- Concurrency control per repo

No cdigo de recuperacin separado.

## Performance Targets

| Mtrica | Target | Justificacin |
| --- | --- | --- |
| Search (cache hit) | < 50ms | LRU cache hit = Map lookup |
| Search (cache miss) | < 500ms | `git log/grep` en 600k commits |
| Diff render (10k lneas) | < 100ms | Virtual list + Canvas minimap |
| Blame annotate (10k lneas) | < 200ms | `git blame` parse streaming |
| Index size | < 100MB | LRU 100 entries = ~10MB |

## Integracin con Features previas

| Feature | Integracin |
| --- | --- |
| 001 Headless | `GitCommand`, `AuditLog`, `RepoLock`, fixtures |
| 002 Graph | `markers.ts` -> graph markers, `affordances.ts` -> UI |
| 003 State | `RepoState` detection -> search filters, diff context |
| 004 Chrome | `SearchBar` en toolbar, `DiffViewer` en sidebar, `BlamePanel` en sidebar |

## Decisiones descartadas

| Opcin | Por qu no |
| --- | --- |
| Watchman incremental index | Dependencia sistema, no en Windows CI, complejidad |
| diff-match-patch | ~10KB, menos control word-level/inline |
| `git diff` + parse | Parse overhead, menos control formatos |
| Canvas puro diff | Seleccin/copy/accessibility complejos |
| SQLite/IndexedDB index | WASM/async overhead, complejidad para beneficio marginal |
| Inline conflict editor | Scope creep, Article VIII |

---

*Research version 1.0.0*