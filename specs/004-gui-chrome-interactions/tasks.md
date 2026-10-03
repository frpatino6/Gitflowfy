---
description: Task breakdown for Feature 004 - GUI Chrome & Interactions
---

# Tasks: GUI Chrome & Interactions

**Branch**: `004-gui-chrome-interactions`
**Plan**: [plan.md](plan.md)
**Status**: Draft

**Constitution Article III applies**: tests first, always. The task order below
encodes that. Do not reorder a test task after its implementation task.

---

## Phase 0: Safety Net

<!-- Integration fixtures from Features 001/002/003 + Tauri app fixtures.
     Nothing else gets built until this passes. -->

- [ ] T001 Create Tauri app fixture builder (`tests/fixtures/app_builder.rs`)
      - real Tauri app with webview, invoke handlers wired to Feature 001 core
      - [FR-001, FR-002, FR-003, FR-004, FR-005, FR-006]
- [ ] T002 Create integration test harness: real Tauri + webview + Feature 001/002/003 cores
      - spawns actual `git` binary via core, compares UI invoke results
      - [NFR-005, NFR-001]
- [ ] T003 [P] Fixture: rebase-in-progress repo (from Feature 001 G3)
      - [Article V, FR-003]
- [ ] T004 [P] Fixture: merge conflict with unmerged index (from Feature 001 G3)
      - [Article V, FR-003]
- [ ] T005 [P] Fixture: detached HEAD repo (from Feature 001 G3)
      - [Article V, FR-003]
- [ ] T006 [P] Fixture: missing reflog directory (from Feature 001 G3)
      - [Article V, FR-003]
- [ ] T007 [P] Fixture: partial clone repo (from Feature 001 G3)
      - [Article V, FR-003]
- [ ] T008 [P] Fixture: sparse checkout repo (from Feature 001 G3)
      - [Article V, FR-003]
- [ ] T009 [P] Fixture: reftable backend repo (from Feature 001 G3)
      - [Article V, FR-003]
- [ ] T010 [P] Fixture: submodule conflict repo (from Feature 001 G3)
      - [Article V, FR-003]

**Gate**: T001-T002 pass. Do not start Phase 1 before this.

---

## Phase 1: Core (Tauri Invoke Surface + Settings + Shortcuts)

<!-- Article IV: every capability reachable headlessly before GUI.
     Each invoke command: test first, then implement. -->

### 1.1 Tauri App Skeleton + Invoke Wiring

- [ ] T011 [NFR-001] Test: `cargo run --bin gitflowfy` launches Tauri window < 2s
      - fails until `apps/desktop/src/main.rs` + `tauri.conf.json` exist
- [ ] T012 [NFR-001] Implement: Tauri app skeleton (`apps/desktop/Cargo.toml`, `tauri.conf.json`, `main.rs`)
      - makes T011 pass
- [ ] T013 [FR-001] Test: `invoke("repo:open", { path })` returns `RepoState` from Feature 001
      - uses fixture from T003, fails until invoke handler exists
- [ ] T014 [FR-001] Implement: `repo:open` handler in `apps/desktop/src/main.rs`
      - calls Feature 001 `GitCommand::open_repo`, returns `RepoState`
      - makes T013 pass
- [ ] T015 [FR-001] Test: `invoke("repo:close", { repoId })` closes repo cleanly
- [ ] T016 [FR-001] Implement: `repo:close` handler
      - makes T015 pass
- [ ] T017 [FR-001] Test: `invoke("git:command", { repoId, args })` returns `Invocation` (audit log)
      - verifies Article I: spawns real `git`, records cwd/argv/exit/duration
- [ ] T018 [FR-001] Implement: `git:command` handler
      - makes T017 pass
- [ ] T019 [FR-003] Test: `invoke("state:detect", { repoId })` returns current `RepoState`
      - detects Article V states via Feature 003
- [ ] T020 [FR-003] Implement: `state:detect` handler
      - makes T019 pass
- [ ] T021 [FR-003] Test: `invoke("state:recover", { repoId, action })` runs recovery command
      - action = "continue" | "abort" | "skip" per Article V state
- [ ] T022 [FR-003] Implement: `state:recover` handler
      - makes T021 pass
- [ ] T023 [FR-002] Test: `invoke("graph:load", { repoId })` returns ArrayBuffer (graph.bin)
      - integrates Feature 002 graph export
- [ ] T024 [FR-002] Implement: `graph:load` handler
      - makes T023 pass

### 1.2 Settings Persistence (JSON + semver + auto-migration)

- [ ] T025 [NFR-006] Test: `invoke("settings:get")` returns default Settings schema v1
      - schema: theme, shortcuts, layout, windowState
- [ ] T026 [NFR-006] Implement: `settings:get` handler + `apps/desktop/src/settings.rs`
      - JSON file at `%APPDATA%/gitflowfy/settings.json` (Win) / `~/.config/gitflowfy/` (Linux/macOS)
      - makes T025 pass
- [ ] T027 [NFR-006] Test: `invoke("settings:set", { key, value })` persists + returns updated
- [ ] T028 [NFR-006] Implement: `settings:set` handler
      - makes T027 pass
- [ ] T029 [NFR-006] Test: settings migration v1 -> v2 (add new field, auto-migrate)
      - writes v1 file, reads v2, asserts migration ran
- [ ] T030 [NFR-006] Implement: migration logic in `settings.rs`
      - makes T029 pass

### 1.3 Keyboard Shortcuts Registry

- [ ] T031 [FR-004] Test: `invoke("shortcuts:get")` returns default ShortcutMap
      - defaults from spec Story 4: Ctrl+S commit, Ctrl+P push, etc.
- [ ] T032 [FR-004] Implement: `shortcuts:get` handler + `apps/desktop/src/shortcuts.rs`
      - Tauri `global-shortcut` plugin registration
      - makes T031 pass
- [ ] T033 [FR-004] Test: `invoke("shortcuts:set", { key, shortcut })` updates + re-registers global
- [ ] T034 [FR-004] Implement: `shortcuts:set` handler
      - makes T033 pass
- [ ] T035 [FR-004] Test: global shortcut fires `invoke("git:command", ...)` for mapped action
      - e.g. Ctrl+S -> `git:command` with ["commit"]
- [ ] T036 [FR-004] Implement: shortcut -> invoke dispatch in `shortcuts.rs`
      - makes T035 pass

### 1.4 Window Management Commands

- [ ] T037 [FR-005] Test: `invoke("window:split", { direction })` splits webview
- [ ] T038 [FR-005] Implement: `window:split` handler in `apps/desktop/src/window.rs`
      - makes T037 pass
- [ ] T039 [FR-005] Test: `invoke("window:tab:new", { repoPath })` creates new tab with repo
- [ ] T040 [FR-005] Implement: `window:tab:new` handler
      - makes T039 pass
- [ ] T041 [FR-005] Test: `invoke("window:tab:close", { tabId })` closes tab
- [ ] T042 [FR-005] Implement: `window:tab:close` handler
      - makes T041 pass
- [ ] T043 [FR-005] Test: `invoke("window:fullscreen", { fullscreen })` toggles fullscreen
- [ ] T044 [FR-005] Implement: `window:fullscreen` handler
      - makes T043 pass

---

## Phase 2: Headless Surface

<!-- Article IV: every invoke command works without a visible window.
     Headless mode = Tauri `webview` headless (no window) or CLI entry. -->

- [ ] T045 [NFR-001] Test: headless mode launches, runs `repo:open`, `git:command`, `state:detect` in < 500ms
      - no window created, uses `tauri::test` or `--headless` flag
- [ ] T046 [NFR-001] Implement: headless entry point in `apps/desktop/src/main.rs`
      - `--headless` flag, runs invoke sequence, exits
      - makes T045 pass
- [ ] T047 [NFR-004] Test: all invoke commands return JSON-serializable shapes (no Rust types leak)
      - verifies `settings:get`, `shortcuts:get`, `graph:load` return clean JSON
- [ ] T048 [NFR-004] Implement: serialize all invoke returns via `serde_json`
      - makes T047 pass
- [ ] T049 [NFR-005] Test: headless suite passes on Windows + Linux (CI matrix)
      - reuses T002 harness, runs without display server

---

## Phase 3: Graph / Render Integration

<!-- Feature 003 markers + affordances integration into webview.
     Article VI benchmark if render path touched. -->

- [ ] T050 [FR-002] Test: `crates/ui/src/graph/markers.tsx` imports Feature 003 types, renders markers
      - uses `graph:load` ArrayBuffer, parses CSR, draws branch/HEAD tags
- [ ] T051 [FR-002] Implement: `markers.tsx` - SolidJS component using Canvas 2D
      - makes T050 pass
- [ ] T052 [FR-003] Test: `crates/ui/src/graph/affordances.tsx` shows recovery buttons for Article V states
      - reads `RepoState.inFlight` from `state:detect`, renders affordance per state
- [ ] T053 [FR-003] Implement: `affordances.tsx` - recovery UI (Continue Rebase, Abort Merge, etc.)
      - buttons call `state:recover` via invoke
      - makes T052 pass
- [ ] T054 [NFR-003] Test: frame p99 < 33.4ms on 600k commit fixture (pan/zoom)
      - headless benchmark harness from Feature 002 spike
- [ ] T055 [NFR-003] Implement: Canvas render optimizations (dirty rects, requestAnimationFrame batching)
      - makes T054 pass
- [ ] T056 Run headless perf benchmark, record numbers in plan.md
      - **Gate**: p99 < 33.4ms, no regression > 10% from Feature 002 baseline

---

## Phase 4: UI Components

<!-- SolidJS components, test-first per component.
     Each component: test (Vitest + Testing Library) then implement. -->

### 4.1 App Shell + Theme

- [ ] T057 [NFR-004] Test: `App` component mounts, applies theme CSS vars, respects `prefers-color-scheme`
      - Vitest + JSDOM, asserts `data-theme` on `:root`
- [ ] T058 [NFR-004] Implement: `crates/ui/src/app.tsx` + `styles/variables.css`
      - CSS custom properties for light/dark, SolidJS `createSignal` for theme
      - makes T057 pass
- [ ] T059 [NFR-004] Test: theme toggle persists via `settings:set`
      - clicks toggle, asserts `settings.json` updated
- [ ] T060 [NFR-004] Implement: theme toggle in `App` + `settings:set` invoke
      - makes T059 pass

### 4.2 Toolbar (Story 1)

- [ ] T061 [FR-001] Test: `Toolbar` renders 7 actions: Commit, Push, Pull, Branch, Merge, Rebase, Stash
      - each button has `data-testid`, click calls correct `invoke`
- [ ] T062 [FR-001] Implement: `crates/ui/src/components/toolbar.tsx`
      - SolidJS `For` over action list, `invoke("git:command", ...)`
      - makes T061 pass
- [ ] T063 [FR-001] Test: Toolbar actions disabled when no repo open
      - `repo:open` not called -> buttons `disabled`
- [ ] T064 [FR-001] Implement: disabled state via `RepoStore` signal
      - makes T063 pass
- [ ] T065 [NFR-004] Test: Toolbar keyboard accessible (Tab, Enter, Space, ARIA labels)
      - `axe-core` scan on mounted component

### 4.3 Menubar (Story 2)

- [ ] T066 [FR-002] Test: native menubar (`tauri-menu`) has all 8 menus with items per spec
      - File, Edit, View, Repository, Branch, Remote, Window, Help
- [ ] T067 [FR-002] Implement: `apps/desktop/src/menu.rs` + Tauri `Menu` API
      - each item emits Tauri event -> webview `invoke`
      - makes T066 pass
- [ ] T068 [FR-002] Test: menubar items enable/disable based on repo state
      - e.g. "Commit" disabled if no repo, "Push" disabled if no upstream
- [ ] T069 [FR-002] Implement: menu state sync via `RepoStore` + Tauri `MenuItem::set_enabled`
      - makes T068 pass
- [ ] T070 [FR-002] Test: keyboard accelerators match spec (Ctrl+S, Ctrl+P, etc.)
      - Tauri `accelerator` on each `MenuItem`

### 4.4 Left Sidebar (Story 3)

- [ ] T071 [FR-003] Test: `SidebarLeft` tabs: Repos, Branches, Tags, Remotes, Stashes
      - each tab renders list from `RepoState`, click emits selection event
- [ ] T072 [FR-003] Implement: `crates/ui/src/components/sidebar-left.tsx`
      - SolidJS `createStore` for active tab, `For` over branches/tags/remotes/stashes
      - makes T071 pass
- [ ] T073 [FR-003] Test: Branches tab - checkout on click calls `git:command` ["checkout", branch]
- [ ] T074 [FR-003] Implement: branch checkout handler in sidebar
      - makes T073 pass
- [ ] T075 [FR-003] Test: Repos tab - shows open repos, click switches active repo
      - calls `repo:open` for new, emits `repoChanged` event
- [ ] T076 [FR-003] Implement: repo switching in sidebar
      - makes T075 pass
- [ ] T077 [NFR-004] Test: sidebar resize handle (drag) works, persists width in settings
      - CSS Grid + custom handle, `settings:set` on drag end

### 4.5 Right Sidebar (Story 3)

- [ ] T078 [FR-003] Test: `SidebarRight` tabs: Diff, Commit Details, File Tree
      - Diff tab shows placeholder until Feature 005
- [ ] T079 [FR-003] Implement: `crates/ui/src/components/sidebar-right.tsx`
      - makes T078 pass
- [ ] T080 [FR-003] Test: Commit Details shows selected commit info (hash, author, date, message)
      - reads from `GraphStore` selected commit
- [ ] T081 [FR-003] Implement: commit details view
      - makes T080 pass
- [ ] T082 [NFR-004] Test: File Tree keyboard nav (arrows, Enter expand/collapse)

### 4.6 Status Bar

- [ ] T083 [FR-001] Test: `StatusBar` shows: current branch, sync status (ahead/behind), repo state badge
      - reads from `RepoState` + `GraphStore`
- [ ] T084 [FR-001] Implement: `crates/ui/src/components/statusbar.tsx`
      - makes T083 pass
- [ ] T085 [FR-003] Test: in-flight state badge (Rebasing, Merging, Conflicts) with click -> affordance
      - integrates `affordances.tsx` inline

### 4.7 Dialogs (Story 1, 6)

- [ ] T086 [FR-001] Test: `CommitDialog` opens on toolbar Commit click / Ctrl+S
      - form: message textarea, file list (staged/unstaged), commit button
- [ ] T087 [FR-001] Implement: `crates/ui/src/components/dialogs/commit.tsx`
      - `invoke("git:command", ["commit", "-m", msg])` on submit
      - makes T086 pass
- [ ] T088 [FR-006] Test: `SettingsDialog` opens on Ctrl+, / menubar
      - tabs: General (theme), Shortcuts (editable), Git (user/email), Advanced
- [ ] T089 [FR-006] Implement: `crates/ui/src/components/dialogs/settings.tsx`
      - shortcuts editor: click key -> press new combo -> `shortcuts:set`
      - makes T088 pass
- [ ] T090 [FR-003] Test: `BranchPicker` dialog (New Branch, Checkout, Delete, Rename)
      - called from toolbar Branch / sidebar Branches tab

### 4.8 Graph View Integration

- [ ] T091 [FR-002] Test: `GraphView` component loads `graph:load` ArrayBuffer, renders with markers + affordances
      - pan/zoom via wheel + drag, 60fps
- [ ] T092 [FR-002] Implement: `crates/ui/src/components/graph-view.tsx`
      - Canvas 2D, `markers.tsx` + `affordances.tsx` composition
      - makes T091 pass
- [ ] T093 [NFR-003] Test: GraphView frame p50 < 16.7ms idle, p99 < 33.4ms pan/zoom (600k fixture)
      - reuses T054 harness

### 4.9 Layout System (Story 3, 5)

- [ ] T094 [NFR-004] Test: CSS Grid layout: toolbar | menubar | sidebar-left | graph | sidebar-right | statusbar
      - resize handles for left/right sidebars, persist sizes in `layout` settings
- [ ] T095 [NFR-004] Implement: `styles/components.css` + `layout.ts` store
      - makes T094 pass
- [ ] T096 [FR-005] Test: split view (horizontal/vertical) via `window:split` invoke
      - creates second GraphView or sidebar, sync selection
- [ ] T097 [FR-005] Implement: split view in `App` + `window.rs` handler
      - makes T096 pass

---

## Phase 5: Integration

<!-- Full suite on Windows + Linux. Constitutional audit. -->

- [ ] T098 [NFR-001] Test: cold launch < 2s (Windows + Linux CI)
      - `t_launch_time` benchmark
- [ ] T099 [NFR-002] Test: memory < 200MB RSS with 600k repo open (Windows + Linux CI)
      - `t_memory_usage` benchmark
- [ ] T100 [NFR-003] Test: 60fps UI - all animations, pan/zoom, sidebar transitions
      - `t_frame_rate` benchmark
- [ ] T101 [NFR-004] Test: `axe-core` CI scan passes (WCAG 2.1 AA)
      - runs on every component mount in test suite
- [ ] T102 [NFR-005] Test: native feel - titlebar, traffic lights (macOS), Mica (Win), GTK (Linux)
      - visual regression via Playwright screenshots
- [ ] T103 [NFR-006] Test: settings persist + migrate across restart
      - `t_settings_persist`, `t_settings_migrate`
- [ ] T104 [FR-004] Test: all shortcuts work globally (app focused + unfocused)
      - `t_shortcuts_work`, `t_shortcuts_persist`
- [ ] T105 [FR-005] Test: tabs - open 3 repos, switch, reorder, close
      - `t_tabs`
- [ ] T106 [FR-005] Test: split view horizontal/vertical, resize, close pane
      - `t_split`
- [ ] T107 [FR-005] Test: fullscreen toggle (F11 + menubar)
      - `t_fullscreen`
- [ ] T108 Full suite green on Windows (CI)
- [ ] T109 [P] Full suite green on Linux (CI)
- [ ] T110 Constitutional audit: no violations, exceptions documented in plan.md
      - Article I: no `git` spawn in `apps/` or `crates/ui/`
      - Article II: no operation log in UI
      - Article VII: 3 crates max
      - Article VIII: no wrapper traits
      - Article IX: real repos in tests

---

## Dependencies

| Task | Depends on | Can run in parallel with |
| --- | --- | --- |
| T003-T010 | T001 | T003-T010 (each other) |
| T011 | T002 | - |
| T013 | T002, T012 | T015, T017, T019, T021, T023 |
| T025 | T012 | T031, T037, T039, T041, T043 |
| T031 | T012 | T025, T037, T039, T041, T043 |
| T037 | T012 | T025, T031, T039, T041, T043 |
| T045 | T014, T018, T020, T022, T024 | - |
| T050 | T024, T048 | T052 |
| T052 | T020, T048 | T050 |
| T054 | T050, T051, T052, T053 | - |
| T057 | T048 | - |
| T061 | T058 | T066, T071, T078, T083 |
| T066 | T012 | T061, T071, T078, T083 |
| T071 | T058 | T061, T066, T078, T083 |
| T078 | T058 | T061, T066, T071, T083 |
| T083 | T058 | T061, T066, T071, T078 |
| T086 | T062 | T088, T090 |
| T088 | T030 | T086, T090 |
| T091 | T024, T051, T053, T055 | - |
| T094 | T058 | - |
| T096 | T038, T094 | - |
| T098-T107 | T056, T093 | - |

---

## Parallel Execution

**Wave 1 (Phase 0 fixtures)**: T003, T004, T005, T006, T007, T008, T009, T010
**Wave 2 (Phase 1 invoke handlers)**: T013, T015, T017, T019, T021, T023
**Wave 3 (Phase 1 settings/shortcuts/window)**: T025, T031, T037, T039, T041, T043
**Wave 4 (Phase 2 headless)**: T045, T047
**Wave 5 (Phase 3 graph integration)**: T050, T052
**Wave 6 (Phase 4 UI components)**: T061, T066, T071, T078, T083
**Wave 7 (Phase 4 dialogs)**: T086, T088, T090
**Wave 8 (Phase 5 integration)**: T108, T109

---

## Progress Log

| Date | Task | Outcome | Notes |
| --- | --- | --- | --- |
| | | | |

---

## Checklist

- [ ] Every task traces to an FR or NFR in spec.md
- [ ] Every test task precedes its implementation task
- [ ] Phase 0 safety net passes before feature work
- [ ] Every in-scope Article V state has a fixture and a test (reused from 001/002/003)
- [ ] Headless surface done before UI (Phase 2 before Phase 4)
- [ ] Perf benchmark re-run if render path touched (Phase 3 gate)
- [ ] Parallel waves contain no hidden dependencies
- [ ] No `git` spawn in `crates/ui/` or `apps/desktop/` (Article I, IV)
- [ ] No operation log in UI (Article II)
- [ ] 3 crates max respected (Article VII)
- [ ] No wrapper traits (Article VIII)
- [ ] Real repos in integration tests (Article IX)