---
description: Fill this template when breaking a plan into executable tasks.
---

# Tasks: [FEATURE NAME]

**Branch**: `[NNN-feature-name] |
**Plan**: [plan.md](plan.md)
**Status**: Draft | Approved | In Progress | Complete

**Constitution Article III applies**: tests first, always. The task order below
encodes that. Do not reorder a test task after its implementation task.

---

## Phase 0: Safety Net

<!-- The characterization tests that prove our tool matches `git` itself.
     Nothing else gets built until this passes: it is the thing that makes
     every later "it works" claim trustworthy. -->

- [ ] T001 Create the fixture repository builder (`tests/fixtures/build.rs`)
      - real repos: linear, merged, octopus, orphan, shallow, partial clone
- [ ] T002 Create the differential test harness that runs an operation through
      both our core and raw `git`, then compares resulting state
- [ ] T003 [P] Fixture: rebase-in-progress, mid-conflict state
- [ ] T004 [P] Fixture: merge conflict with unmerged index
- [ ] T005 [P] Fixture: detached HEAD
- [ ] T006 [P] Fixture: missing reflog directory

**Gate**: T001-T002 pass. Do not start Phase 1 before this.

---

## Phase 1: Core

<!-- Each task: one behavior, test first, traceable to an FR. -->

- [ ] T007 [FR-001] Test: <behavior> - fails first
- [ ] T008 [FR-001] Implement: <behavior> - makes T007 pass
- [ ] T009 [P] [FR-002] Test: <behavior>
- [ ] T010 [P] [FR-002] Implement: <behavior>

---

## Phase 2: Headless Surface

<!-- Article IV: the capability must work with no window before any GUI work. -->

- [ ] T011 Test: CLI command produces expected output
- [ ] T012 Implement: CLI command
- [ ] T013 [P] Test: JSON output shape
- [ ] T014 [P] Implement: JSON serialization

---

## Phase 3: Graph / Render

<!-- Only if the render path is touched. Article VI benchmark required. -->

- [ ] T015 Test: frame time p99 under budget on 600k fixture
- [ ] T016 Implement: <render change>
- [ ] T017 Run headless perf benchmark, record numbers in plan.md

**Gate**: p99 < 33.4 ms. No regression > 10% from baseline.

---

## Phase 4: UI

- [ ] T018 [P] Test: component behavior
- [ ] T019 [P] Implement: component
- [ ] T020 [P] Test: keyboard navigation
- [ ] T021 [P] Implement: keyboard navigation

---

## Phase 5: Integration

- [ ] T022 Full suite green on Windows
- [ ] T023 [P] Full suite green on Linux
- [ ] T024 Constitutional audit: no violations, exceptions documented

---

## Dependencies

| Task | Depends on | Can run in parallel with |
| --- | --- | --- |
| T003 | T001 | T004, T005, T006 |
| T007 | T002 | - |

## Parallel Execution

**Wave 1**: T003, T004, T005, T006
**Wave 2**: T007
**Wave 3**: T009, T013
**Wave 4**: T011, T016
**Wave 5**: T018, T020

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
