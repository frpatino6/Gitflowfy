---
description: Fill this template when breaking a plan into executable tasks.
---

# Tasks: Repository State Recovery

**Branch**: `003-repository-state-recovery`
**Plan**: [plan.md](plan.md)
**Status**: Draft

**Constitution Article III applies**: tests first, always. The task order below
encodes that. Do not reorder a test task after its implementation task.

---

## Phase 0: Safety Net

<!-- The characterization tests that prove our tool matches `git` itself.
     Nothing else gets built until this passes: it is the thing that makes
     every later "it works" claim trustworthy. -->

- [ ] T001 Create fixture repository builder for all 10 Article V states (`tests/fixtures/build.rs`)
      - Reuses Feature 001 fixtures (rebase-in-progress, merge-conflict, cherry-pick, revert, bisect)
      - Adds 4 new fixtures: partial-clone, sparse-checkout, missing-reflog, reftable, submodule-conflict
      - Each fixture: creates repo, puts it in exact state, records expected RepoState variant
- [ ] T002 Create differential test harness for state detection (`tests/harness/state_detect.rs`)
      - Runs `detect_state()` on each fixture
      - Compares against ground truth from `git status --porcelain=v2` + filesystem probes
      - Asserts exact RepoState enum variant + payload matches
- [ ] T003 [P] Fixture: partial-clone (promisor remote + filter)
- [ ] T004 [P] Fixture: sparse-checkout (pattern active)
- [ ] T005 [P] Fixture: missing-reflog (logs/ directory absent)
- [ ] T006 [P] Fixture: reftable-backend (extensions.refStorage=reftable)
- [ ] T007 [P] Fixture: submodule-conflict (submodule with unmerged index)

**Gate**: T001-T002 pass. Do not start Phase 1 before this.

---

## Phase 1: Core Detection & Recovery

<!-- Each task: one behavior, test first, traceable to an FR/NFR. -->

### Detection Types & Enum

- [ ] T008 [FR-001, NFR-002] Test: `RepoState` enum has 11 variants (Clean + 10 states) with correct payloads
      - File: `crates/core/src/state/types.rs`
      - Asserts: variant names match plan table, payloads typed correctly
- [ ] T009 [FR-001, NFR-002] Implement: `RepoState` enum + `StateInfo` struct in `types.rs`
- [ ] T010 [FR-001, NFR-001] Test: `detect_state(repo_path)` returns correct variant for each fixture
      - File: `crates/core/src/state/detect.rs`
      - 11 sub-tests (Clean + 10 states)
      - Asserts: detection < 100ms on 600k repo (NFR-001)
- [ ] T011 [FR-001, NFR-001, NFR-004] Implement: `detect_state()` in `detect.rs`
      - Uses filesystem probes + `git status --porcelain=v2` + config reads
      - Priority order: rebase > merge > cherry-pick > revert > bisect > partial > sparse > missing-reflog > reftable > submodule
      - Returns `RepoState` enum

### Recovery Actions

- [ ] T012 [FR-002, NFR-003] Test: `recover_action(repo, action)` returns correct `GitCommand` for each state/action
      - File: `crates/core/src/state/recover.rs`
      - 16 sub-tests (actions per plan table rows)
      - Asserts: command argv matches plan exactly, uses `GitCommand` from Feature 001
- [ ] T013 [FR-002, NFR-003] Implement: `recover_action()` in `recover.rs`
      - Maps `RecoveryAction` enum -> `GitCommand` per plan table
      - MissingReflog/Reftable: returns `WarnOnly`/`InfoOnly` (no command)
- [ ] T014 [FR-002, NFR-003] Test: recovery action executes via `GitCommand`, audit log records invocation
      - Spawns command, captures exit code, asserts audit entry written
- [ ] T015 [FR-002, NFR-003] Implement: wire `recover_action()` -> `GitCommand.execute()` -> audit log

### Module Wiring

- [ ] T016 [FR-001, FR-002] Test: `crates/core/src/state/mod.rs` exports `detect_state`, `recover_action`, `RepoState`, `RecoveryAction`
- [ ] T017 [FR-001, FR-002] Implement: `mod.rs` public exports

---

## Phase 2: Headless Surface (CLI)

<!-- Article IV: the capability must work with no window before any GUI work. -->

- [ ] T018 [FR-001, NFR-004] Test: `gitflowfy state detect <repo>` prints JSON with correct RepoState
      - Runs binary against each fixture
      - Asserts: JSON shape matches `StateInfo` schema, exit code 0
- [ ] T019 [FR-001, NFR-004] Implement: `state detect` CLI command in `crates/core/src/cli.rs`
      - Calls `detect_state()`, serializes to JSON via `serde_json`
- [ ] T020 [FR-002, NFR-003] Test: `gitflowfy state recover <repo> <action>` executes action, prints result JSON
      - Asserts: command runs via `GitCommand`, audit entry created, exit code propagated
- [ ] T021 [FR-002, NFR-003] Implement: `state recover` CLI command
      - Parses `RecoveryAction` from arg, calls `recover_action()`, executes, prints JSON result
- [ ] T022 [P] [NFR-004] Test: CLI works on Windows + Linux (CI matrix)
- [ ] T023 [P] [NFR-004] Implement: cross-platform path handling in CLI args

---

## Phase 3: Graph / Render Integration

<!-- Article VI benchmark required. Reuses Feature 002 renderer. -->

- [ ] T024 [FR-003, NFR-005] Test: graph markers render for each RepoState variant
      - File: `crates/ui/src/state/markers.ts`
      - Loads each fixture in headless renderer, captures frame
      - Asserts: visual markers present per spec acceptance scenarios
- [ ] T025 [FR-003, NFR-005] Implement: `markers.ts` - maps `RepoState` -> visual markers
      - Merge conflict: red commit + tooltip with files
      - Orphan branches: dotted components
      - Detached HEAD: detached icon
      - Rebase in progress: "REBASE" badge on recent commits
      - Bisect: highlight current commit + range
- [ ] T026 [NFR-005] Test: graph marker render time < 16ms frame budget (Article VI)
      - Runs headless perf benchmark on 600k fixture with markers enabled
- [ ] T027 [NFR-005] Implement: optimize marker rendering if needed

---

## Phase 4: UI Affordances

<!-- TypeScript in crates/ui, reuses Feature 002 webview stack. -->

### Detection Display

- [ ] T028 [FR-001] Test: `state:detect` invoke returns `RepoState` from core
      - File: `crates/ui/src/state/detect.ts`
      - Mocks Tauri invoke, asserts payload shape
- [ ] T029 [FR-001] Implement: `detect.ts` - `invoke("state:detect", { repo })` -> `RepoState`

### Recovery Affordances (Buttons + Display)

- [ ] T030 [FR-001, FR-002] Test: `affordances.ts` renders correct UI for each RepoState
      - File: `crates/ui/src/state/affordances.ts`
      - 11 sub-tests (Clean + 10 states)
      - Asserts: state name, affected files, recovery buttons with exact git command labels
      - Asserts: semantic HTML buttons, ARIA labels, keyboard focusable
- [ ] T031 [FR-001, FR-002] Implement: `affordances.ts` - React/Preact component per state
      - Clean: "Repository clean" message
      - In-progress states: state name + files + Continue/Abort buttons
      - Bisect: Good/Bad/Skip/Reset buttons
      - Partial clone: Unshallow button
      - Sparse checkout: Disable button
      - Missing reflog: warning + button (with confirmation)
      - Reftable: info only
      - Submodule conflict: Update Recursive button
- [ ] T032 [FR-002] Test: clicking recovery button invokes `state:recover` with correct action
      - Asserts: invoke called with `RecoveryAction`, result handled (success/error toast)
- [ ] T033 [FR-002] Implement: button click handlers -> `invoke("state:recover", { repo, action })`
- [ ] T034 [P] [NFR-004] Test: keyboard navigation (Tab/Enter/Space) works on all affordance buttons
- [ ] T035 [P] [NFR-004] Implement: focus management, ARIA live regions for status updates
- [ ] T036 [P] [FR-001] Test: UI polls/detects state change after recovery action
- [ ] T037 [P] [FR-001] Implement: auto-refresh detection after recovery completes

### Module Wiring

- [ ] T038 [FR-001, FR-002, FR-003] Test: `crates/ui/src/state/mod.ts` exports detect, affordances, markers
- [ ] T039 [FR-001, FR-002, FR-003] Implement: `mod.ts` public exports

---

## Phase 5: Integration

- [ ] T040 Full test suite green on Windows (CI)
- [ ] T041 [P] Full test suite green on Linux (CI)
- [ ] T042 Constitutional audit: no Article I-IX violations, exceptions documented in plan.md
- [ ] T043 Performance gate: run Article VI benchmark, record numbers in plan.md
      - Detect < 100ms on 600k repo
      - Frame p99 < 33.4ms with markers
- [ ] T044 End-to-end test: open each fixture in desktop app, verify detect + display + recover flow

---

## Dependencies

| Task | Depends on | Can run in parallel with |
| --- | --- | --- |
| T003-T007 | T001 | T003, T004, T005, T006, T007 |
| T008 | T002 | - |
| T010 | T002, T009 | - |
| T012 | T002, T009 | - |
| T014 | T013 | - |
| T018 | T011 | - |
| T020 | T015 | - |
| T024 | T009, T011 | - |
| T028 | T011 | - |
| T030 | T009 | T030, T032, T034 |
| T032 | T015 | T030, T034 |
| T040 | T017, T021, T027, T039 | T041 |

---

## Parallel Execution

**Wave 1 (Phase 0 fixtures)**: T003, T004, T005, T006, T007
**Wave 2 (Phase 1 detection tests)**: T008, T010, T012
**Wave 3 (Phase 1 detection impl)**: T009, T011, T013
**Wave 4 (Phase 1 recovery tests)**: T014, T016
**Wave 5 (Phase 1 recovery impl)**: T015, T017
**Wave 6 (Phase 2 CLI tests)**: T018, T020, T022
**Wave 7 (Phase 2 CLI impl)**: T019, T021, T023
**Wave 8 (Phase 3 graph tests)**: T024, T026
**Wave 9 (Phase 3 graph impl)**: T025, T027
**Wave 10 (Phase 4 UI tests)**: T028, T030, T032, T034, T036
**Wave 11 (Phase 4 UI impl)**: T029, T031, T033, T035, T037
**Wave 12 (Phase 4 wiring)**: T038, T039
**Wave 13 (Phase 5 integration)**: T040, T041, T042, T043, T044

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