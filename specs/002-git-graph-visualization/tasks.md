---
description: Fill this template when breaking a plan into executable tasks.
---

# Tasks: Git Graph Visualization

**Branch**: `002-git-graph-visualization`
**Plan**: [plan.md](plan.md)
**Status**: Draft

**Constitution Article III applies**: tests first, always. The task order below
encodes that. Do not reorder a test task after its implementation task.

---

## Phase 0: Safety Net

<!-- The characterization tests that prove our graph builder matches `git` itself.
     Nothing else gets built until this passes: it is the thing that makes
     every later "it works" claim trustworthy. -->

- [ ] T001 Create differential test harness for graph builder (`tests/graph/diff_harness.rs`)
      - Runs `git log --all --topo-order --format=...` through our builder and raw `git`, compares OIDs + topology
      - Reuses Feature 001 fixture repos (linear, merged, octopus, orphan, detached, empty, path-edge)
- [ ] T002 [P] Fixture validation: verify Feature 001 fixture OIDs are stable (`tests/graph/fixture_validate.rs`)
      - Asserts `git rev-list --all --topo-order` output matches recorded OIDs for all 7 fixtures
- [ ] T003 [P] Test: graph builder on empty repo produces valid `graph.bin` with n=0 (NFR-001)
- [ ] T004 [P] Test: graph builder on linear fixture matches `git rev-list` exactly (Story 4, NFR-001)
- [ ] T005 [P] Test: graph builder on merged fixture has correct parent_count=2 at merge commit (Story 4, NFR-001)
- [ ] T006 [P] Test: graph builder on octopus fixture has parent_count=3+ at octopus merge (Story 4, NFR-001)
- [ ] T007 [P] Test: graph builder on orphan fixture produces 2 disconnected components (Story 4, NFR-001)
- [ ] T008 [P] Test: graph builder on detached HEAD fixture shows detached commit correctly (Story 4, NFR-001)
- [ ] T009 [P] Test: graph builder on path-edge fixture handles spaces/non-ASCII in refs (Story 4, NFR-001)

**Gate**: T001-T002 pass. Do not start Phase 1 before this.

---

## Phase 1: Core Graph Builder

<!-- Each task: one behavior, test first, traceable to an FR/NFR. -->

- [ ] T010 [Story 1] Test: `GraphBuilder::build(repo)` returns `graph.bin` bytes matching spike format v1 (NFR-002)
- [ ] T011 [Story 1] Implement: `GraphBuilder::build()` in `crates/core/src/graph/builder.rs`
      - Spawns `git log --all --topo-order --format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s` via `GitCommand`
      - Streams stdout, parses unit-separated fields, builds CSR arrays
- [ ] T012 [Story 1] Test: `GraphBuilder::build()` extracts 600k commits in < 30 s (NFR-001, Article VI)
- [ ] T013 [Story 1] Test: `graph.bin` size < 32 MB for 600k commits (NFR-001, Article VI)
- [ ] T014 [Story 1] Implement: CSR lane assignment algorithm (port exact from spike `build-graph.mjs`)
      - Free-lane pool, O(n) lane assignment, Int32Array output
- [ ] T015 [Story 1] Test: lane assignment produces valid lanes (no overlaps, parent lanes visible) (NFR-001)
- [ ] T016 [Story 1] Test: `graph.bin` binary format v1 header + 5 typed arrays + string pool (NFR-002)
- [ ] T017 [Story 1] Implement: `format.rs` encode/decode for `graph.bin` v1
      - Magic `0x47524150`, version, commit count, edge count, 5 CSR arrays, string pool
- [ ] T018 [Story 1] Test: decode(encode(graph)) round-trip preserves all arrays byte-for-byte (NFR-002)
- [ ] T019 [Story 1] Test: `GraphBuilder::build()` includes refs from `git for-each-ref` in string pool (Story 1)
- [ ] T020 [Story 1] Implement: refs extraction and embedding in `graph.bin` string pool
- [ ] T021 [Story 1] Test: graph builder on repo with 0 commits returns valid empty `graph.bin` (Story 4)
- [ ] T022 [Story 1] Test: graph builder propagates `git log` non-zero exit codes, no partial output (Story 1)

---

## Phase 2: Headless Surface (CLI)

<!-- Article IV: the capability must work with no window before any GUI work. -->

- [ ] T023 [Story 1, NFR-003] Test: `gitflowfy graph build <repo> <out>` writes `graph.bin` to file
- [ ] T024 [Story 1, NFR-003] Implement: CLI `graph build` subcommand in `crates/core/src/cli/bin/gitflowfy.rs`
      - Calls `GraphBuilder::build()`, writes bytes to output path
- [ ] T025 [Story 1, NFR-003] Test: `gitflowfy graph load <repo>` outputs `graph.bin` bytes to stdout
- [ ] T026 [Story 1, NFR-003] Implement: CLI `graph load` subcommand
      - Calls `GraphBuilder::build()`, writes bytes to stdout (for piping / Tauri invoke)
- [ ] T027 [NFR-003] Test: CLI `graph build` exit codes: 0 success, 1 repo error, 2 git error, 3 IO error
- [ ] T028 [NFR-003] Test: CLI `graph build --help` shows usage with repo and output args

---

## Phase 3: Renderer (TypeScript in crates/ui)

<!-- Only if the render path is touched. Article VI benchmark required. -->

- [ ] T029 [Story 2] Test: `decodeGraph(Uint8Array)` returns typed arrays matching `graph.bin` v1 (NFR-002)
- [ ] T030 [Story 2] Implement: `decode.ts` zero-copy decode using `bytemuck` (port from spike)
- [ ] T031 [Story 2] Test: decode round-trip: encode -> decode preserves all arrays (NFR-002)
- [ ] T032 [Story 2] Test: renderer initial load of 600k graph < 2 s (NFR-001, Article VI)
- [ ] T033 [Story 2] Implement: `renderer.ts` Canvas 2D virtualized render (port exact from spike `public/index.html`)
      - Visible commits only, lane rectangles, commit dots, ref labels
- [ ] T034 [Story 2] Test: frame p50 <= 16.7 ms during horizontal pan (NFR-001, Article VI)
- [ ] T035 [Story 2] Test: frame p99 <= 33.4 ms during horizontal pan (NFR-001, Article VI)
- [ ] T036 [Story 2] Test: max frame <= 50 ms during zoom (NFR-001, Article VI)
- [ ] T037 [Story 2] Implement: `viewport.ts` virtualized pan/zoom with transform matrix
- [ ] T038 [Story 2] Test: viewport culls commits outside visible rect (performance)
- [ ] T039 [Story 2] Test: renderer handles empty graph (n=0) without crash (Story 4)
- [ ] T040 [Story 2] Test: renderer handles corrupted `graph.bin` with descriptive error (NFR-002)
- [ ] T041 [Story 2] Run headless perf benchmark (`npm run bench` in `crates/ui`), record numbers in plan.md

**Gate**: p99 < 33.4 ms. No regression > 10% from spike baseline (19.5 ms).

---

## Phase 4: Tauri Integration

<!-- Article IV satisfied: headless works, now GUI wires to it. -->

- [ ] T042 [Story 3] Test: `invoke("graph:load", { repo })` returns `ArrayBuffer` matching `graph.bin` (NFR-003)
- [ ] T043 [Story 3] Implement: Tauri command `graph:load` in `apps/desktop/src/main.rs`
      - Calls core `GraphBuilder::build()`, returns `Vec<u8>` -> Tauri `ArrayBuffer`
- [ ] T044 [Story 3] Test: Tauri app launches, webview loads, renders graph from `invoke("graph:load")`
- [ ] T045 [Story 3] Implement: `apps/desktop/src/main.rs` Tauri v2 setup
      - Webview with `crates/ui` renderer, `invoke` handler for `graph:load`
- [ ] T046 [Story 3] Test: pan/zoom in Tauri webview meets frame budgets (NFR-001)
- [ ] T047 [Story 3] Test: `graph:load` invoke timeout 30s, error propagated to UI (Error Handling)
- [ ] T048 [Story 3] Implement: webview entry `crates/ui/src/main.ts` calls `invoke("graph:load")`, decodes, renders
- [ ] T049 [Story 3] Test: Tauri `graph:build` CLI subcommand works identically to headless (NFR-003)
- [ ] T050 [Story 3] Implement: Tauri shell command for `graph:build` (reuses CLI binary)

---

## Phase 5: Integration & Cross-Platform

- [ ] T051 Full test suite green on Windows (CI)
- [ ] T052 [P] Full test suite green on Linux (CI)
- [ ] T053 Constitutional audit: no violations, exceptions documented in plan.md Complexity Tracking
- [ ] T054 [P] Benchmark regression check: extract < 30s, graph < 32MB, load < 2s, p50 < 16.7ms, p99 < 33.4ms, max < 50ms
- [ ] T055 [P] Binary format forward compat: renderer v1 reads v1 + v2 (when v2 exists) (NFR-002)
- [ ] T056 [P] Verify no `git` spawn outside `crates/core` (gate G4 script)
- [ ] T057 [P] Verify no embedded Git crate in dependency tree (gate G5 script)
- [ ] T058 [P] Verify workspace has exactly 3 crates: core, ui, desktop (Article VII)

---

## Dependencies

| Task | Depends on | Can run in parallel with |
| --- | --- | --- |
| T002 | T001 | T003-T009 |
| T003-T009 | T001, T002 | each other |
| T010 | T001, T002 | - |
| T011 | T010 | - |
| T012 | T011 | T013 |
| T013 | T011 | T012 |
| T014 | T011 | - |
| T015 | T014 | - |
| T016 | T011 | T017 |
| T017 | T016 | - |
| T018 | T017 | - |
| T019 | T011 | T020 |
| T020 | T019 | - |
| T021 | T011 | T022 |
| T022 | T011 | T021 |
| T023 | T011 | T025 |
| T024 | T023 | - |
| T025 | T011 | T023 |
| T026 | T025 | - |
| T027 | T024 | T028 |
| T028 | T024 | T027 |
| T029 | T017 | T031 |
| T030 | T029 | - |
| T031 | T030 | T029 |
| T032 | T030 | T034, T035, T036 |
| T033 | T030 | - |
| T034 | T033 | T032, T035, T036 |
| T035 | T033 | T032, T034, T036 |
| T036 | T033 | T032, T034, T035 |
| T037 | T033 | - |
| T038 | T037 | - |
| T039 | T033 | T040 |
| T040 | T030 | T039 |
| T041 | T034, T035, T036 | - |
| T042 | T026, T030 | T044 |
| T043 | T042 | - |
| T044 | T043, T048 | - |
| T045 | T043 | T048 |
| T046 | T044 | - |
| T047 | T043 | - |
| T048 | T030, T043 | T045 |
| T049 | T024 | T050 |
| T050 | T049 | - |
| T051 | T044, T046 | T052, T054-T058 |
| T052 | T044, T046 | T051, T054-T058 |
| T053 | T051, T052 | - |
| T054 | T051, T052 | T051, T052, T055-T058 |
| T055 | T031 | T054, T056-T058 |
| T056 | T024, T043 | T054, T055, T057, T058 |
| T057 | - | T054-T056, T058 |
| T058 | - | T054-T057 |

---

## Parallel Execution

**Wave 1 (Phase 0 fixtures)**: T003, T004, T005, T006, T007, T008, T009
**Wave 2 (Phase 1 core tests)**: T012, T013, T015, T016, T018, T019, T021, T022
**Wave 3 (Phase 1 implementations)**: T014, T017, T020 (after their test deps)
**Wave 4 (Phase 2 CLI tests)**: T023, T025, T027, T028
**Wave 5 (Phase 2 CLI impls)**: T024, T026 (after test deps)
**Wave 6 (Phase 3 renderer tests)**: T029, T031, T032, T034, T035, T036, T039, T040
**Wave 7 (Phase 3 renderer impls)**: T030, T033, T037 (after test deps)
**Wave 8 (Phase 4 Tauri tests)**: T042, T044, T046, T047, T049
**Wave 9 (Phase 4 Tauri impls)**: T043, T045, T048, T050 (after test deps)
**Wave 10 (Phase 5 integration)**: T051, T052, T054, T055, T056, T057, T058 (T053 after T051/T052)

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
- [ ] Every in-scope Article V state has a fixture and a test (reused from Feature 001)
- [ ] Headless surface done before UI (Phase 2 before Phase 4)
- [ ] Perf benchmark re-run if render path touched (Phase 3 gate)
- [ ] Parallel waves contain no hidden dependencies