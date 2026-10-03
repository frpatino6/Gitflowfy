---
description: Fill this template when breaking a plan into executable tasks.
---

# Tasks: Advanced Interactions

**Branch**: `005-advanced-interactions`
**Plan**: [plan.md](plan.md)
**Status**: Draft

**Constitution Article III applies**: tests first, always. The task order below
encodes that. Do not reorder a test task after its implementation task.

---

## Phase 0: Safety Net

<!-- Differential harness for search/diff/blame operations.
     Reuses fixtures from Feature 001/003. Nothing else starts until this passes. -->

- [ ] T001 Verify Feature 001 differential harness passes (G1, G2, G3)
      - `cargo test -p gitflowfy-core test_differential_harness`
      - Fixtures: linear, merged, octopus, orphan, shallow, partial clone, rebase-in-progress, merge-conflict, detached-head, missing-reflog
- [ ] T002 Extend differential harness for search operations
      - Test: `search:query` vs `git log --grep` + `git grep` parity
      - Test: `blame:annotate` vs `git blame -L` parity
      - Test: `diff:compare` vs `git diff` parity
- [ ] T003 [P] Fixture: repo with 600k commits (llvm-project subset) for perf tests
- [ ] T004 [P] Fixture: repo with binary files + large files for diff/blame edge cases
- [ ] T005 [P] Fixture: repo with submodules for multi-repo dashboard test
- [ ] T006 [P] Fixture: repo in rebase-in-progress + merge-conflict for search/diff/blame in odd states

**Gate**: T001-T002 pass. Do not start Phase 1 before this.

---

## Phase 1: Core

<!-- Search, Diff, Blame core logic. One behavior per task, test first, traceable to FR. -->

### Search Core (Story 1, 2 / NFR-001, NFR-003)

- [ ] T007 [FR-001] Test: SearchQuery parsing - regex, fuzzy, filters (author, date, file, hash)
      - Input: query string -> SearchQuery struct with parsed filters
      - Fails first: no implementation exists
- [ ] T008 [FR-001] Implement: SearchQuery parser in `crates/core/src/search/query.rs`
- [ ] T009 [FR-001] Test: LRU cache hit/miss behavior - 100 entries max, eviction order
      - Insert 101 entries -> first evicted
      - Cache key = hash(query + filters + repo)
- [ ] T010 [FR-001] Implement: LRU cache in `crates/core/src/search/index.rs`
- [ ] T011 [FR-001] Test: `git log --all --grep` execution + parsing -> SearchResults
      - Filters: author, since, until, paths
      - Returns: commit hash, short message, author, date, files changed
- [ ] T012 [FR-001] Implement: `run_git_log_search` in `crates/core/src/search/index.rs`
- [ ] T013 [FR-002] Test: `git grep -n` execution + parsing -> FileSearchResults
      - Returns: file path, line number, matched line, context lines
      - Filter by extension (-- "*.rs")
- [ ] T014 [FR-002] Implement: `run_git_grep_search` in `crates/core/src/search/index.rs`
- [ ] T015 [FR-001, FR-002] Test: Combined search (commits + files) with scope toggle
      - Scope: current repo vs all repos
      - Results merged, deduplicated, sorted by relevance
- [ ] T016 [FR-001, FR-002] Implement: `search_combined` in `crates/core/src/search/mod.rs`
- [ ] T017 [NFR-001] Test: Search perf - cache miss < 500ms on 600k commits (T003 fixture)
      - Query: "fix bug" -> results < 500ms
- [ ] T018 [NFR-003] Test: Search index memory < 100MB (100 LRU entries)
      - Measure heap after 100 cached searches

### Diff Core (Story 5 / NFR-002)

- [ ] T019 [FR-005] Test: Myers O(ND) diff algorithm - basic equal/delete/insert
      - Input: ["a", "b", "c"], ["a", "x", "c"] -> EditScript with delete "b", insert "x"
      - Property test: diff(a,b) inverted == diff(b,a) inverted
- [ ] T020 [FR-005] Implement: Myers O(ND) in `crates/core/src/diff/myers.rs`
- [ ] T021 [FR-005] Test: Word-level diff - split lines by word boundaries
      - "hello world" vs "hello there world" -> word insert "there"
- [ ] T022 [FR-005] Implement: Word-level diff in `crates/core/src/diff/format.rs`
- [ ] T023 [FR-005] Test: Diff formatting - side-by-side, inline, ignore whitespace
      - Side-by-side: two columns, synchronized scroll markers
      - Inline: unified diff with +/- prefixes
      - Ignore whitespace: treat "a  b" == "a b"
- [ ] T024 [FR-005] Implement: DiffFormat enum + formatters in `crates/core/src/diff/format.rs`
- [ ] T025 [FR-005] Test: `diff:compare` core command - base/target commits -> DiffResult
      - Uses Myers + formatter
      - Returns EditScript + metadata (file stats, binary detection)
- [ ] T026 [FR-005] Implement: `diff_compare` in `crates/core/src/diff/mod.rs`
- [ ] T027 [NFR-002] Test: Diff render perf < 100ms for 10k lines (T004 fixture)
      - Measure core diff computation only (not DOM)

### Blame Core (Story 3)

- [ ] T028 [FR-003] Test: `git blame -L start,end file` parsing -> BlameAnnotations
      - Per line: commit hash, author, date, summary
      - Handles: moved lines (-M), copied lines (-C), ignored whitespace (-w)
- [ ] T029 [FR-003] Implement: `blame_annotate` in `crates/core/src/blame/annotate.rs`
- [ ] T030 [FR-003] Test: File history timeline - `git log --oneline -- file` -> HistoryEntry[]
      - Each entry: commit hash, date, author, message, lines changed
- [ ] T031 [FR-003] Implement: `file_history` in `crates/core/src/blame/mod.rs`
- [ ] T032 [FR-003] Test: Heatmap data - line age calculation (recent=red, old=blue)
      - Input: BlameAnnotations -> age per line in days
      - Output: color stops for gradient

### Stash Core (Story 6)

- [ ] T033 [FR-006] Test: `stash:list` -> StashEntry[] (index, message, date, branch, commit)
      - Parses `git stash list --format="%gd %gs %cr %gd"`
- [ ] T034 [FR-006] Implement: `stash_list` in `crates/core/src/cli/stash.rs`
- [ ] T035 [FR-006] Test: Stash actions - apply, pop, drop, rename, branch
      - `git stash apply <n>`, `git stash pop <n>`, `git stash drop <n>`
      - `git stash rename <n> <name>`, `git stash branch <name> <n>`
- [ ] T036 [FR-006] Implement: `stash_action` in `crates/core/src/cli/stash.rs`

### Worktree Core (Story 7)

- [ ] T037 [FR-007] Test: `worktree:list` -> WorktreeEntry[] (path, branch, commit, locked, prunable)
      - Parses `git worktree list --porcelain`
- [ ] T038 [FR-007] Implement: `worktree_list` in `crates/core/src/cli/worktree.rs`
- [ ] T039 [FR-007] Test: Worktree actions - add, remove, move, prune
      - `git worktree add <path> <branch>`, `git worktree remove <path>`
      - `git worktree move <old> <new>`, `git worktree prune`
- [ ] T040 [FR-007] Implement: `worktree_action` in `crates/core/src/cli/worktree.rs`

### Remote Core (Story 9)

- [ ] T041 [FR-009] Test: `remote:list` -> RemoteEntry[] (name, url, fetch_refs, push_refs)
      - Parses `git remote -v` + `git config --get-regexp remote.`
- [ ] T042 [FR-009] Implement: `remote_list` in `crates/core/src/cli/remote.rs`
- [ ] T043 [FR-009] Test: Remote actions - add, remove, edit, fetch, prune, push-tags
      - `git remote add <name> <url>`, `git remote remove <name>`
      - `git remote set-url <name> <url>`, `git fetch --all --prune`, `git push --tags`
- [ ] T044 [FR-009] Implement: `remote_action` in `crates/core/src/cli/remote.rs`

### Commit Actions Core (Story 8)

- [ ] T045 [FR-008] Test: Commit range actions - cherry-pick, revert, bisect, tag
      - `git cherry-pick A..B`, `git revert A..B`
      - `git bisect start/good/bad/skip/reset` + graph highlight data
      - `git tag -a <tag> -m <msg>`, `git tag -d <tag>`, `git push --tags`
- [ ] T046 [FR-008] Implement: `commit_actions` + `tag_action` in `crates/core/src/cli/commit_actions.rs`

---

## Phase 2: Headless Surface

<!-- Article IV: CLI commands for every capability before GUI. -->

- [ ] T047 Test: `gitflowfy search --repo <path> --query "fix" --author "John" --json`
      - Output: JSON SearchResults, exit 0
- [ ] T048 Implement: `search` subcommand in `crates/core/src/cli/search.rs`
- [ ] T049 Test: `gitflowfy blame --repo <path> --file src/main.rs --lines 10,20 --json`
      - Output: JSON BlameAnnotations
- [ ] T050 Implement: `blame` subcommand in `crates/core/src/cli/blame.rs`
- [ ] T051 Test: `gitflowfy diff --repo <path> --base HEAD~1 --target HEAD --format side-by-side --json`
      - Output: JSON DiffResult
- [ ] T052 Implement: `diff` subcommand in `crates/core/src/cli/diff.rs`
- [ ] T053 Test: `gitflowfy stash list --repo <path> --json`
- [ ] T054 Implement: `stash` subcommand (list/action) in `crates/core/src/cli/stash.rs`
- [ ] T055 Test: `gitflowfy worktree list --repo <path> --json`
- [ ] T056 Implement: `worktree` subcommand in `crates/core/src/cli/worktree.rs`
- [ ] T057 Test: `gitflowfy remote list --repo <path> --json`
- [ ] T058 Implement: `remote` subcommand in `crates/core/src/cli/remote.rs`
- [ ] T059 Test: `gitflowfy commit cherry-pick --repo <path> --range A..B --json`
- [ ] T060 Implement: `commit` subcommand in `crates/core/src/cli/commit_actions.rs`
- [ ] T061 Test: `gitflowfy tag create --repo <path> --name v1.0 --msg "Release" --json`
- [ ] T062 Implement: `tag` subcommand in `crates/core/src/cli/tag.rs`
- [ ] T063 [P] Test: All CLI commands --help output valid
- [ ] T064 [P] Test: CLI JSON output schema matches TypeScript types in `crates/ui/src/types/`

---

## Phase 3: Graph / Render

<!-- Diff viewer render path. Article VI benchmark required. -->

- [ ] T065 Test: DiffViewer virtual list renders 10k lines < 16ms frame (p99)
      - Headless: mount DiffViewer, measure frame time via requestAnimationFrame
      - Data: T004 fixture (10k line diff)
- [ ] T066 Implement: DiffViewer virtualized DOM + Canvas minimap in `crates/ui/src/diff/DiffViewer.tsx`
- [ ] T067 Test: DiffViewer minimap click -> scroll to position
- [ ] T068 Implement: Minimap interaction in DiffViewer
- [ ] T069 Test: DiffViewer keyboard navigation (j/k, up/down, home/end, search)
- [ ] T070 Implement: Keyboard navigation in DiffViewer
- [ ] T071 [NFR-002] Run headless perf benchmark: diff render 10k lines < 100ms
      - Record numbers in plan.md Performance Impact table
      - `cargo test -p gitflowfy-core test_diff_perf -- --nocapture`

**Gate**: p99 < 33.4 ms (Article VI). No regression > 10% from Feature 004 baseline.

---

## Phase 4: UI

<!-- Components for each feature. Test-first: component behavior, then implementation. -->

### Search UI (Story 1, 2)

- [ ] T072 Test: SearchPanel - query input, filter chips, results list, scope toggle
      - Type query -> debounced invoke -> results render
      - Click result -> navigate to commit/file
- [ ] T073 Implement: SearchPanel in `crates/ui/src/search/SearchPanel.tsx`
- [ ] T074 Test: SearchBar component - global search bar with repo scope toggle
      - Keyboard: Cmd/Ctrl+K focus, Esc close
      - "All Repos" toggle -> scope changes
- [ ] T075 Implement: SearchBar in `crates/ui/src/components/SearchBar.tsx`
- [ ] T076 Test: SearchIndex client-side LRU cache (100 entries)
      - Hits instant, misses invoke core
- [ ] T077 Implement: SearchIndex in `crates/ui/src/search/SearchIndex.ts`

### Diff UI (Story 5)

- [ ] T078 Test: DiffFormat component - toggle side-by-side/inline/word-level/ignore-ws
      - Each toggle re-renders with new format
- [ ] T079 Implement: DiffFormat in `crates/ui/src/diff/DiffFormat.tsx`
- [ ] T080 Test: Syntax highlighting in diff lines (shiki/wasm)
      - Language detection from file extension
- [ ] T081 Implement: Syntax highlighting in DiffLine component

### Blame UI (Story 3)

- [ ] T082 Test: BlamePanel - line annotations overlay, click commit -> open in graph
      - Toggle blame on/off
      - Heatmap toggle -> lines colored by age
- [ ] T083 Implement: BlamePanel in `crates/ui/src/blame/BlamePanel.tsx`
- [ ] T084 Test: HistoryView - file commit timeline, click -> show diff
      - Virtualized list for many commits
- [ ] T085 Implement: HistoryView in `crates/ui/src/blame/HistoryView.tsx`

### Multi-Repo Dashboard (Story 4)

- [ ] T086 Test: Dashboard grid - repo cards with branch, ahead/behind, changes, stashes, fetch status
      - Click card -> focus repo in main window
      - "Fetch All" button -> invoke fetch on all repos
- [ ] T087 Implement: Dashboard in `crates/ui/src/dashboard/Dashboard.tsx`

### Stash UI (Story 6)

- [ ] T088 Test: StashList - list with actions (apply, pop, drop, rename, branch)
      - Rename: inline edit
      - Branch from stash: dialog -> invoke
- [ ] T089 Implement: StashList in `crates/ui/src/stash/StashList.tsx`

### Worktree UI (Story 7)

- [ ] T090 Test: WorktreeList - list with actions (add, remove, move, prune)
      - Add: dialog with path + branch selector
      - Move: drag-drop or dialog
- [ ] T091 Implement: WorktreeList in `crates/ui/src/worktree/WorktreeList.tsx`

### Remote UI (Story 9)

- [ ] T092 Test: RemoteManager - list with actions (add, edit, remove, fetch, prune, push-tags)
      - Add/edit: dialog with name + URL
- [ ] T093 Implement: RemoteManager in `crates/ui/src/remote/RemoteManager.tsx`

### Commit Actions UI (Story 8)

- [ ] T094 Test: Cherry-pick range dialog - select range A..B, confirm -> invoke
- [ ] T095 Implement: CherryPickDialog in `crates/ui/src/components/CherryPickDialog.tsx`
- [ ] T096 Test: Revert range dialog - select range, confirm -> invoke
- [ ] T097 Implement: RevertDialog in `crates/ui/src/components/RevertDialog.tsx`
- [ ] T098 Test: Bisect UI - good/bad/skip/reset buttons + graph highlight
      - Visual: current bisect commit highlighted in graph
- [ ] T099 Implement: BisectPanel in `crates/ui/src/components/BisectPanel.tsx`
- [ ] T100 Test: Tag management - create (name + msg), delete, push
- [ ] T101 Implement: TagDialog in `crates/ui/src/components/TagDialog.tsx`

### Accessibility (NFR-005)

- [ ] T102 Test: axe-core audit on all new components - zero violations AA
- [ ] T103 Implement: ARIA labels, roles, keyboard focus management in all components
- [ ] T104 Test: Keyboard-only navigation for all panels (Tab, Arrow keys, Enter, Esc)
- [ ] T105 Implement: Focus traps, skip links, live regions for dynamic content

---

## Phase 5: Integration

<!-- Full suite on Windows + Linux. Constitutional audit. -->

- [ ] T106 Full test suite green on Windows (CI)
      - `cargo test --workspace`
      - `pnpm test` (UI unit + e2e)
- [ ] T107 [P] Full test suite green on Linux (CI)
- [ ] T108 Constitutional audit: no violations, exceptions documented in plan.md
      - Article I: no git spawn outside core
      - Article II: no operation log
      - Article VII: <= 3 crates
      - Article VIII: no wrapper traits
      - Article IX: integration tests use real git
- [ ] T109 Perf regression check: search < 500ms, diff < 100ms, index < 100MB on 600k fixture
- [ ] T110 Update plan.md with measured numbers from benchmarks

---

## Dependencies

| Task | Depends on | Can run in parallel with |
| --- | --- | --- |
| T007 | T002 | - |
| T009 | T008 | T011, T013 |
| T011 | T008 | T009, T013 |
| T013 | T008 | T009, T011 |
| T015 | T012, T014 | - |
| T019 | T002 | - |
| T021 | T020 | T023 |
| T023 | T020 | T021 |
| T025 | T020, T024 | - |
| T028 | T002 | - |
| T030 | T028 | - |
| T032 | T028 | - |
| T033 | T002 | T037, T041, T045 |
| T037 | T002 | T033, T041, T045 |
| T041 | T002 | T033, T037, T045 |
| T045 | T002 | T033, T037, T041 |
| T047 | T016 | T049, T051, T053, T055, T057, T059, T061 |
| T065 | T026, T024 | - |
| T072 | T016, T048 | T074, T076 |
| T078 | T026, T052 | - |
| T082 | T028, T050 | T084 |
| T086 | T016, T048 | - |
| T088 | T034, T054 | - |
| T090 | T038, T056 | - |
| T092 | T042, T058 | - |
| T094 | T046, T060 | T096, T098, T100 |

---

## Parallel Execution

**Wave 1 (Phase 0 fixtures)**: T003, T004, T005, T006
**Wave 2 (Search core independent)**: T009, T011, T013 (after T008)
**Wave 3 (Diff core independent)**: T021, T023 (after T020)
**Wave 4 (Blame core independent)**: T030, T032 (after T028)
**Wave 5 (Stash/Worktree/Remote/Commit core independent)**: T033, T037, T041, T045 (after T002)
**Wave 6 (CLI independent)**: T047, T049, T051, T053, T055, T057, T059, T061 (after respective core impl)
**Wave 7 (UI independent components)**: T072, T074, T076, T078, T082, T084, T086, T088, T090, T092, T094, T096, T098, T100 (after respective core+CLI)
**Wave 8 (A11y)**: T102, T104 (after all UI components)
**Wave 9 (Integration)**: T106, T107

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
- [ ] Every in-scope Article V state has a fixture and a test
- [ ] Headless surface done before UI
- [ ] Perf benchmark re-run if render path touched
- [ ] Parallel waves contain no hidden dependencies