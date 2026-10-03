# Implementation Plan: GUI Chrome & Interactions

**Branch**: `004-gui-chrome-interactions`
**Spec**: [spec.md](spec.md)
**Status**: Draft
**Created**: 2026-09-30

---

## Phase -1: Pre-Implementation Gates

### Constitution Check

| Article | Binding here? | How this plan satisfies it |
| --- | --- | --- |
| I - Shell out | **S** | UI llama `GitCommand` via Tauri `invoke`. No spawn directo. |
| II - Reflog only | **S** | UI no mantiene estado repo. Solo refleja `RepoState` de Feature 003. |
| III - Test first | **S** | Phase 0: component tests antes que implementation. |
| IV - CLI first | **S** | Toda capacidad expuesta via `invoke` (headless) antes que UI. |
| V - Odd repo states | **S** | UI muestra `RepoState` de Feature 003 + affordances. |
| VI - Perf budgets | **S** | NFR-001: launch < 2s, NFR-002: mem < 200MB, NFR-003: 60fps. |
| VII - <=3 crates | **S** | Aade `crates/ui` + `apps/desktop`. Total = 3/3. |
| VIII - Anti-abstraction | **S** | SolidJS signals + stores. Sin Redux/Zustand. Tauri `invoke` directo. |
| IX - Integration first | **S** | Tests contra Tauri real + webview real. Fixtures Feature 001/002/003. |

### Gates

| Gate | Pass Criteria |
| --- | --- |
| G1 | `git version` parseable |
| G2 | Feature 001 G2 passed (overhead budget) |
| G3 | Feature 001 G3 passed (fixtures) |
| G4 | No `git` spawn fuera de `crates/core` |
| G5 | No `libgit2`/`gix`/`jgit` |

---

## Technical Approach

### Stack Decision

**Elegido**:
- **UI Framework**: SolidJS (fine-grained signals, ~7KB, no VDOM)
- **State**: SolidJS Signals + Stores
- **Theme**: CSS Custom Properties + `prefers-color-scheme`
- **Layout**: CSS Grid + Custom Resize Handles
- **Settings**: JSON + semver + auto-migration
- **Build**: Vite (dev), Tauri bundler (release)

**Rationale**: 
- SolidJS: signals fine-grained = 60fps sin re-renders. ~7KB gzipped. TypeScript first-class.
- CSS vars: zero runtime, nativo, funciona con SolidJS.
- CSS Grid: nativo, 0 deps. Custom resize handles = control total.
- JSON + semver + auto-migration: nativo, migracin automtica.

### Architecture

```
crates/ui/
  package.json
  tsconfig.json
  vite.config.ts
  src/
    main.ts              # Entry point webview
    app.tsx              # Root component
    components/
      toolbar.tsx        # Toolbar principal
      menubar.tsx        # Menubar nativa (tauri-menu)
      sidebar-left.tsx   # Repos, Branches, Tags, Remotes, Stashes
      sidebar-right.tsx  # Diff, CommitDetails, FileTree
      statusbar.tsx      # Status bar
      dialogs/
        commit.tsx       # Commit dialog
        settings.tsx     # Settings dialog
        branch.tsx       # Branch picker
    state/
      repos.ts           # Store: repos abiertos, active repo
      theme.ts           # Signal: theme (light/dark/system)
      shortcuts.ts       # Store: shortcuts map
      layout.ts          # Store: window layout (split, tabs)
    graph/
      markers.tsx        # Feature 003 markers integration
      affordances.tsx    # Feature 003 affordances integration
    utils/
      invoke.ts          # Tauri invoke wrappers
      shortcuts.ts       # Keyboard shortcuts registry
    styles/
      variables.css      # CSS custom properties
      global.css         # Reset + base
      components.css     # Component styles

apps/desktop/
  Cargo.toml
  tauri.conf.json
  src/
    main.rs              # Tauri setup, invoke handlers
    menu.rs              # Native menubar (tauri-menu)
    window.rs            # Window management (tabs, split, fullscreen)
    settings.rs          # Settings persistence (JSON + migration)
    shortcuts.rs         # Global shortcuts registration
```

### Data Flow

1. **Launch**: Tauri `main.rs` -> crea ventana -> carga `crates/ui` en webview
2. **Init**: `main.ts` -> `app.tsx` -> registra shortcuts, carga settings, detecta repo
3. **Repo open**: `invoke("repo:open", { path })` -> Feature 001 `GitCommand` -> `RepoState` -> UI updates
4. **User action**: Click toolbar -> `invoke("git:commit", ...)` -> Feature 001 `GitCommand` -> audit -> UI update
5. **Shortcut**: Global shortcut -> `invoke("git:commit", ...)` -> same as click

### Tauri Invoke Surface

| Command | Payload | Returns |
| --- | --- | --- |
| `repo:open` | `{ path: string }` | `{ repoState: RepoState }` |
| `repo:close` | `{ repoId: string }` | `void` |
| `git:command` | `{ repoId, args: string[] }` | `Invocation` |
| `state:detect` | `{ repoId }` | `RepoState` |
| `state:recover` | `{ repoId, action }` | `Invocation` |
| `graph:load` | `{ repoId }` | `ArrayBuffer` (graph.bin) |
| `settings:get` | `{}` | `Settings` |
| `settings:set` | `{ key, value }` | `void` |
| `shortcuts:get` | `{}` | `ShortcutMap` |
| `shortcuts:set` | `{ key, shortcut }` | `void` |
| `window:split` | `{ direction: 'horizontal'|'vertical' }` | `void` |
| `window:tab:new` | `{ repoPath }` | `tabId` |
| `window:tab:close` | `{ tabId }` | `void` |
| `window:fullscreen` | `{ fullscreen: boolean }` | `void` |

---

## Requirement Traceability

| Requirement | Technical decision | Verified by |
| --- | --- | --- |
| Story 1: Window + Toolbar | `apps/desktop/src/window.rs` + `toolbar.tsx` | `t_window_opens`, `t_toolbar_actions` |
| Story 2: Menubar | `menu.rs` (tauri-menu) | `t_menubar_complete` |
| Story 3: Sidebars | `sidebar-left.tsx`, `sidebar-right.tsx` | `t_sidebar_navigation`, `t_sidebar_sync` |
| Story 4: Shortcuts | `shortcuts.ts` + `shortcuts.rs` | `t_shortcuts_work`, `t_shortcuts_persist` |
| Story 5: Window Mgmt | `window.rs` (tabs, split, fullscreen) | `t_tabs`, `t_split`, `t_fullscreen` |
| Story 6: Settings | `settings.rs` (JSON + migration) | `t_settings_persist`, `t_settings_migrate` |
| NFR-001: Launch < 2s | Vite + Tauri optimized | `t_launch_time` |
| NFR-002: Mem < 200MB | Rust + SolidJS minimal | `t_memory_usage` |
| NFR-003: 60fps | SolidJS signals + Canvas | `t_frame_rate` |
| NFR-004: a11y | Semantic HTML + ARIA | `axe-core` CI |
| NFR-005: Cross-platform | Tauri v2 native | CI Windows + Linux |
| NFR-006: Settings persist | JSON + semver + auto-migrate | `t_settings_migrate` |

---

## Performance Impact

| Metric | Budget | New expected | Measured |
| --- | --- | --- | --- |
| Cold launch | < 2 s | < 1.5 s | `t_launch_time` |
| Memory (600k repo) | < 200 MB | < 150 MB | `t_memory_usage` |
| Frame p50 (idle) | < 16.7 ms | < 10 ms | `t_frame_p50` |
| Frame p99 (pan/zoom) | < 33.4 ms | < 20 ms | `t_frame_p99` |
| Shortcut latency | < 50 ms | < 20 ms | `t_shortcut_latency` |

---

## Project Structure

```
crates/ui/
  package.json
  tsconfig.json
  vite.config.ts
  src/
    main.ts
    app.tsx
    components/
    state/
    graph/
    utils/
    styles/
  index.html

apps/desktop/
  Cargo.toml
  tauri.conf.json
  src/
    main.rs
    menu.rs
    window.rs
    settings.rs
    shortcuts.rs
```

**Dependencies new**:
- `crates/ui`: `solid-js`, `@tauri-apps/api`, `vite` (dev)
- `apps/desktop`: `tauri` v2, `tauri-plugin-shell`, `tauri-plugin-fs`, `tauri-plugin-dialog`, `tauri-plugin-clipboard-manager`, `tauri-plugin-global-shortcut`

---

## Complexity Tracking

| Exception | Article | Justification | Revisit when |
| --- | --- | --- | --- |
| SolidJS | VIII | Fine-grained reactivity para 60fps. No VDOM. No wrapper. | Si SolidJS MSRV/license conflict. |
| Vite | VIII | Dev server + bundler estndar. No wrapper. | Si Vite MSRV/license conflict. |
| `tauri-plugin-*` | VIII | Plugins oficiales Tauri. No wrapper. | Si Tauri API breaking change. |
| `crates/ui` + `apps/desktop` | VII | Constitucin permite 3 crates total. | Nunca; arquitectura fijada. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| R1: SolidJS learning curve | Media | Delay inicial | Docs excelentes, equipo pequeo, learning curve bajo. |
| R2: Tauri v2 breaking changes | Baja | Rework | Tauri v2 estable. Pin version. |
| R3: Webview2 en Linux CI | Media | Tests fallan | WebKitGTK fallback. CI configura ambos. |
| R4: Memory > 200MB | Baja | Fail NFR-002 | Profile en CI. SolidJS + Rust minimal. |
| R3: a11y regression | Media | WCAG fail | `axe-core` en CI. Tests manuales. |

---

## Plan Self-Review

- [x] Every requirement traced to decision + test
- [x] All 5 clarifications resolved in spec
- [x] Every git command spelled out literally (via invoke)
- [x] Headless surface defined before GUI (Phase 1 before Phase 2)
- [x] Fixtures for all in-scope states (reused from 001/002/003)
- [x] Perf budget stated + measurement (NFR-001..006)
- [x] Complexity exceptions justified (4, each with trigger)
- [x] Pure ASCII output

---

*Plan version 1.0.0 - listo para revisin constitucional*