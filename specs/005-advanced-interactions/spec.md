---
description: Feature 005 - Advanced interactions: search/filter across commits/files, blame/history per file, multi-repo dashboard, commit/file search, diff viewer, stash management.
---

# Feature Specification: Advanced Interactions

**Feature Branch**: `005-advanced-interactions`
**Status**: Draft
**Created**: 2026-09-30
**Constitution**: `.claude/constitution.md` v1.1.0

## Purpose

Features 001-004 entregan: core headless, grafo, estados, chrome. Feature 005 aade **interacciones avanzadas** que usuarios power esperan: bsqueda global, blame/history por archivo, multi-repo dashboard, diff viewer avanzado, stash management, worktrees.

## Scope

### In Scope

- **Global Search**: Buscar commits por mensaje, autor, archivo, hash (regex, fuzzy)
- **File Search**: Buscar archivos por nombre, contenido (grep), path
- **Blame/History per File**: Anotacin lnea por lnea, historia del archivo, heatmap
- **Multi-Repo Dashboard**: Vista unificada de mltiples repos, sync status, ahead/behind
- **Advanced Diff Viewer**: Side-by-side, inline, word-level, ignore whitespace, syntax highlight
- **Stash Management**: List, apply, pop, drop, rename, branch from stash
- **Worktree Management**: List, add, remove, move, prune
- **Commit Actions Avanzados**: Cherry-pick range, revert range, bisect UI, tag management
- **Remote Management**: Add/remove/edit remotes, fetch/prune, push tags

### Out of Scope

- Code review / PR workflow (GitHub/GitLab integration)
- CI/CD integration
- Plugin system
- Team collaboration features

## User Stories

### Story 1: Global Search (P0)

**As a** usuario con repo grande
**I want** buscar commits/archivos instantneamente
**so that** encuentre cambios sin navegar manualmente

**Independent test**: Index 600k commits, search "fix bug" -> resultados < 200ms, fuzzy match, filtros (autor, fecha, archivo, hash).

**Acceptance scenarios**:
1. Given repo 600k commits, when search "bug", then resultados < 200ms, highlight matches
2. When filter by autor "John", then solo commits de John
3. When filter by archivo "src/main.rs", then solo commits tocando ese archivo
4. When search por hash corto "abc123", then navega directo al commit

### Story 2: File Search + Grep (P0)

**As a** usuario
**I want** buscar archivos por nombre y contenido
**so that** navegue cdigo sin terminal

**Independent test**: Repo 600k commits, search "fn main" -> resultados < 500ms, syntax highlight, navega a lnea.

**Acceptance scenarios**:
1. When search "fn main", then lista archivos + lneas coincidentes, syntax highlight
2. When click resultado, then abre archivo en diff viewer en esa lnea
3. When filter por extensin ".rs", then solo archivos Rust

### Story 3: Blame / History per File (P0)

**As a** usuario viendo archivo
**I want** ver quin cambi cada lnea y cundo
**so that** entienda evolucin del cdigo

**Independent test**: `git blame -L 10,20 src/main.rs` -> anotacin lnea por lnea, click commit -> historia archivo.

**Acceptance scenarios**:
1. Given archivo abierto, when toggle blame, then cada lnea muestra autor, fecha, commit hash
2. When click commit en blame, then abre commit en grafo + diff
3. When toggle heatmap, then lneas coloreadas por edad (reciente=rojo, viejo=azul)

### Story 4: Multi-Repo Dashboard (P0)

**As a** usuario con mltiples repos
**I want** vista unificada de todos mis repos
**so that** vea estado global sin cambiar ventana

**Independent test**: 5 repos abiertos, dashboard muestra: repo name, branch actual, ahead/behind, uncommitted changes, stashes, fetch status.

**Acceptance scenarios**:
1. Given 5 repos, when open dashboard, then grid con cards por repo
2. When click repo, then focus ese repo en ventana principal
3. When repo tiene fetch pending, then badge "fetch available"
4. When click "Fetch All", then `git fetch --all` en todos

### Story 5: Advanced Diff Viewer (P0)

**As a** usuario revisando cambios
**I want** diff side-by-side, inline, word-level, syntax highlight
**so that** revise cambios eficientemente

**Independent test**: Diff 2 commits grandes, toggle side-by-side/inline, word-level, ignore whitespace, syntax highlight.

**Acceptance scenarios**:
1. Given diff, when toggle side-by-side, then vista side-by-side sincronizada
2. When toggle word-level, then cambios palabra a palabra resaltados
3. When toggle ignore whitespace, then ignora cambios solo whitespace
4. When syntax highlight on, then colorea por lenguaje

### Story 6: Stash Management (P1)

**As a** usuario con stashes
**I want** list, apply, pop, drop, rename, branch from stash
**so that** gestione trabajo en progreso

**Independent test**: 5 stashes, list muestra mensaje/fecha/branch, apply/pop/drop/rename/branch from stash.

**Acceptance scenarios**:
1. Given stashes, when click apply, then `git stash apply`, audit log
2. When click branch from stash, then `git stash branch <name>`, nueva branch
3. When rename stash, then `git stash rename <n> <new-name>`

### Story 7: Worktree Management (P1)

**As a** usuario con worktrees
**I want** list, add, remove, move, prune
**so that** trabaje en mltiples branches simultneamente

**Independent test**: 3 worktrees, list muestra path/branch/commit, add/remove/move/prune.

**Acceptance scenarios**:
1. Given repo, when add worktree, then `git worktree add <path> <branch>`
2. When remove worktree, then `git worktree remove <path>`
3. When prune, then `git worktree prune`

### Story 8: Commit Actions Avanzados (P1)

**As a** usuario
**I want** cherry-pick range, revert range, bisect UI, tag management
**so that** gestione historia avanzada

**Independent test**: Cherry-pick range A..B, revert range A..B, bisect UI (good/bad/skip/reset + graph highlight), tag create/delete/push.

**Acceptance scenarios**:
1. Given commits A..B, when cherry-pick range, then `git cherry-pick A..B`
2. When revert range, then `git revert A..B`
3. When tag create, then `git tag -a <tag> -m <msg>`

### Story 9: Remote Management (P1)

**As a** usuario
**I want** add/remove/edit remotes, fetch/prune, push tags
**so that** gestione remotos sin terminal

**Independent test**: List remotes con URL/fetch refs, add/edit/remove, fetch/prune, push tags.

**Acceptance scenarios**:
1. Given repo, when add remote, then `git remote add <name> <url>`
2. When fetch all, then `git fetch --all --prune`
3. When push tags, then `git push --tags`

## Non-Functional Requirements

### NFR-001: Search Index < 500ms

Index incremental (watchman/fs.watch) o on-demand. Search < 500ms en 600k commits.

### NFR-002: Diff Viewer < 100ms

Diff render < 100ms para archivos < 10k lneas. Virtualizado.

### NFR-003: Search Index Size < 100MB

Index en memoria < 100MB para 600k commits.

### NFR-004: Cross-Platform

Windows/Linux/macOS. CI ambos.

### NFR-005: Accessibility

WCAG 2.1 AA en todos los componentes nuevos.

## Constraints & Decisions

### Q1: Search Index Strategy -> **Resuelto: Hybrid on-demand + LRU cache (C)**

**Decisin**: On-demand `git log/grep` + LRU cache en memoria (ltimo 100 searches). Sin watchman.

**Justificacin**: Watchman (B) aade dependencia sistema, complejidad CI. On-demand (A) simple pero lento en repeat searches. LRU cache (C) = best of both: primer search paga costo, repeats instantneos. 100 entries = ~10MB memoria.

### Q2: Diff Algorithm -> **Resuelto: Myers O(ND) portado a TS (A)**

**Decisin**: Implementar Myers O(ND) en TypeScript para diff viewer. Control total, sin deps.

**Justificacin**: diff-match-patch (B) ~10KB extra, menos control. `git diff` + parse (C) delega a Git pero parse overhead y menos control en word-level/inline/side-by-side. Myers propio = control total, 0 deps, ~200 lneas.

### Q3: Search Index Storage -> **Resuelto: En memoria LRU cache (A)**

**Decisin**: LRU cache en memoria (Map/Trie) para ltimos 100 searches. Sin persistencia.

**Justificacin**: Search es on-demand + cache. Persistencia (B)/(C) aade complejidad (WASM/IndexedDB) para beneficio marginal. 100 entries LRU = ~10MB, acceptables. Si usuario quiere persistir, feature posterior.

### Q4: Diff Viewer Architecture -> **Resuelto: DOM virtualizado + Canvas minimap (B + A hybrid)**

**Decisin**: DOM virtualizado (virtual list) para diff lines + Canvas minimap para overview.

**Justificacin**: Canvas puro (A) para diff text = complejidad seleccin/copy/accessibility. DOM virtualizado (B) = nativo selection/copy/accessibility, virtualizacin = performance. Canvas minimap para overview = best of both.

### Q5: Search Scope Default -> **Resuelto: Repo actual + toggle "All Repos" (A + toggle)**

**Decisin**: Default = repo actual. Toggle "All Repos" en search bar para buscar en todos los repos abiertos.

**Justificacin**: Default seguro (repo actual). Toggle explcito para multi-repo. Configurable (C) aade complejidad settings sin necesidad.

---

## Clarification Check

- [x] Q1 resuelto
- [x] Q2 resuelto
- [x] Q3 resuelto
- [x] Q4 resuelto
- [x] Q5 resuelto

---

*Spec version 1.0.0 - all clarifications resolved, ready for review*