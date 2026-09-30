# Tasks: Headless Git Operations

**Branch**: `001-headless-git-operations`
**Plan**: [plan.md](plan.md)
**Spec**: [spec.md](spec.md)
**Status**: Draft

**Constitution Article III applies**: tests first, always. The order below encodes
that. No test task may be reordered after its implementation task.

---

## Deviations from the sdd-tasks agent's generic instructions

Two, plus one correction made during constitutional review. All three are recorded
here because the constitution makes the spec the tiebreaker, and in each case the
spec's position prevailed.

**1. The Article V fixture list is narrower than the agent's default.** The agent
instructs that every in-scope unusual state gets a fixture in Phase 0, listing rebase,
merge conflict, missing reflog, partial clone, sparse checkout, reftable and orphan.
The spec for this feature scopes fixtures to six baseline shapes and explicitly
defers the rest. The spec wins. The deferred states still carry a binding narrow
obligation from the spec's Edge Cases table -- return Git's exit code, record it, do
not hang, do not corrupt -- and **T022 tests exactly that obligation** for the
deferred set. The full recovery affordances belong to the repository-state feature,
not here.

**2. Phase 0 contains no product code, and that is deliberate.** The agent's Phase 0
is "fixture repos plus the differential harness, nothing else starts until it
passes". Taken literally that is circular: the harness runs operations *through our
surface*, so the surface would have to exist first. The resolution is that Phase 0
uses a **test-only** git runner -- a helper that calls `git` directly from test code
-- to build fixtures and to read observables. The product surface is not written
until Phase 1a, and the harness from Phase 0 tests it the moment it lands.

This is better than the order `plan.md` was written in, which had the surface before
the fixtures. The plan's ordering would have let the harness depend on the code under
test. Flagging it as a genuine improvement over the plan rather than a deviation from
it.

**3. The audit log is persisted, and FR-027 and FR-028 are new requirements added
during review.** The plan originally kept the log in memory, which is incompatible
with Story 2: an in-memory log dies with its process, so a fresh `gitflowfy audit`
would have returned an empty list every time, and "auditable after the fact" would
have been unfalsifiable. The spec was amended rather than the plan being rationalized,
because the requirement was wrong, not the reasoning. Persisting does not breach
Article II: the record holds four invocation fields and no repository state. Section
1e is the decomposition of the corrected requirement.

---

## Phase 0: Safety Net

> No product code in this phase. Test-only helpers plus the CI enforcement scripts.
> The point is that when Phase 1 lands, the thing testing it was not built by it.

- [ ] T001 Create the test-only git runner (`tests/support/git.rs`): a helper that
      spawns `git` with pinned env and returns exit code, stdout bytes, stderr bytes.
      Assert `git version` parses and record it. **Gate G1.**
- [ ] T002 [FR-012] Test: every test run records the git version and platform.
      Fails first -- nothing prints it yet.
- [ ] T003 [FR-011] Test: the same fixture built in two different directories yields
      byte-identical commit OIDs. **Expected to fail on first run.** This is
      **Gate G3**, the gate the whole harness depends on.
- [ ] T004 [FR-011] Implement: the pinning policy -- identity, both dates, TZ,
      `LC_ALL=C`, `autocrlf=false`, `eol=lf`, `ignorecase=false`, `gpgsign=false`,
      and a per-commit incrementing author date so identical-content commits do not
      collapse to one OID. Makes T003 pass.
- [ ] T005 [P] [FR-010] Test: linear fixture -- one branch, no merges, every commit
      exactly one parent. (`tests/fixtures/linear.rs`)
- [ ] T006 [P] [FR-010] Test: merged fixture -- one merge commit, exactly two
      parents, both reachable. (`tests/fixtures/merged.rs`)
- [ ] T007 [P] [FR-010] Test: octopus fixture -- a merge commit with three or more
      parents, all reachable. (`tests/fixtures/octopus.rs`)
- [ ] T008 [P] [FR-010] Test: orphan fixture -- no merge base between the branches.
      (`tests/fixtures/orphan.rs`)
- [ ] T009 [P] [FR-010] Test: detached fixture -- `symbolic-ref -q HEAD` exits 1 and
      `rev-parse HEAD` succeeds. (`tests/fixtures/detached.rs`)
- [ ] T010 [P] [FR-026] Test: empty fixture -- `rev-parse HEAD` exits 128 with Git's
      own unborn-HEAD text **and** `symbolic-ref -q HEAD` exits 0 naming a branch that
      does not exist. Both facts together, per risk R9. (`tests/fixtures/empty.rs`)
- [ ] T011 [P] [FR-010, Edge Cases] Test: a fixture whose **repository path contains
      spaces and a non-ASCII character** builds, and `GitCommand` passes the path
      verbatim to git without escaping or normalization. (`tests/fixtures/path-edge.rs`)
- [ ] T012 Implement: the seven fixture builders, one file each
      (`src/fixtures/linear.rs` and siblings), so T005-T011 can land in parallel.
- [ ] T013 [FR-008] Test: the refs observable -- `for-each-ref` output compared
      byte-for-byte between two repositories. (`tests/observables/refs.rs`)
- [ ] T014 [P] [FR-008] Test: the HEAD observable distinguishes all **three** states
      -- attached, attached-unborn, detached. (`tests/observables/head.rs`)
- [ ] T015 [P] [FR-008] Test: the index observable, `ls-files -s`, distinguishes a
      conflicted index (stages 1/2/3) from a clean one. (`tests/observables/index.rs`)
- [ ] T016 [P] [FR-008] Test: the working-tree observable -- `status --porcelain=v2`
      plus content hashes of every untracked path. Two trees differing only in bytes
      must compare unequal. (`tests/observables/worktree.rs`)
- [ ] T017 [P] [FR-008] Test: the reflog observable uses `%H %gs` and never the raw
      line, so two runs seconds apart compare equal while a real divergence does not.
      (`tests/observables/reflog.rs`)
- [ ] T018 [FR-009] Test: a deliberate divergence in one observable is reported by
      name, with both values, and is not reported as a generic mismatch.
- [ ] T019 Implement: the five observables and the comparison that names the one
      that differed. Makes T013-T018 pass.
- [ ] T020 [FR-007] Test: the differential harness runs one operation through a
      caller-supplied runner twice, from two identical fixtures, and compares.
- [ ] T021 [FR-023] Test: the harness uses real repositories on both sides and
      detects a swap -- if a fixture is mutated between the two runs, the harness
      fails. This is the test that proves the harness is not self-confirming.
- [ ] T022 Implement: the differential harness. Makes T020-T021 pass.
- [ ] T023 [Article V] Test: for each **deferred** in-scope state -- rebase in
      progress, merge conflict, cherry-pick in progress, revert in progress, bisect
      in progress, partial clone, sparse checkout, missing reflog -- an operation
      against it returns Git's own exit code, does not hang, and leaves the
      repository as Git left it. A partial clone is built against a **local path**
      promisor remote, since no fixture may need the network. Reftable is conditional
      on git >= 2.45 and reports **skipped-with-reason** when unavailable, never a
      silent pass.
- [ ] T024 [NFR-002] Test: measure the chosen stack's wrapper overhead -- 200
      invocations dispatched through `GitCommand` minus 200 invoked directly --
      and record median and p99. **This is Gate G2**, and it decides whether the
      5 ms budget is reachable before anything is built on it. Audit writes are
      **not** stubbed out; see T073.
- [ ] T025 [Article I] Create `scripts/check-no-git-outside-core.ps1` and `.sh`:
      fail the build on any `Command::new("git")`, `process.spawn(`, or `exec("git`
      outside `crates/core`. **Gate G4.**
- [ ] T026 [Article I] Create `scripts/check-no-embedded-git.sh`: `cargo tree
      --all-features` must contain no `libgit2`, `git2`, `gix`, or `jgit`.
      **Gate G5.**
- [ ] T027 [NFR-001] Create `scripts/check-perf-scope.sh`: assert the graph and
      renderer modules are untouched by this feature, so "claims no performance
      number" is a checked statement.

**Gate**: T003-T004 pass (**G3 determinism**), T020-T022 pass (**the harness
detects divergence**), T025-T026 exist and pass. Do not start Phase 1 before this.

---

## Phase 1: Core

> The product surface lands here, and the Phase 0 harness tests it immediately.
> Every test is written, observed failing, and only then implemented.

### 1a: The spawn primitive

- [ ] T028 [FR-001, FR-003, FR-023] Test: a real git command returns exit code,
      stdout bytes, stderr bytes, and a positive duration, matching a direct
      invocation. Fails first -- no product code exists.
- [ ] T029 Implement: `src/git/command.rs`, `GitCommand { repo, args }` ->
      `Invocation`. The one and only spawn site. stdin closed, `GIT_PAGER` removed,
      `GIT_TERMINAL_PROMPT=0`, `LC_ALL=C`. Makes T028 pass.
- [ ] T030 [FR-002] Test: an argument containing `;` and `&&` reaches git as one
      literal argument. Proves there is no shell anywhere in the path.
- [ ] T031 [FR-025] Test: a command outside the documented list -- `notes --ref=todo
      list` -- runs successfully. Proves there is no private allowlist.
- [ ] T032 [FR-024] Test: no module reads `.git/objects`. Asserted by a file-level
      check, since it is a structural property rather than a runtime one.
- [ ] T033 Implement: whatever T030-T032 require of the command module. No new
      abstractions (Article VIII) -- a test failure here means the test is wrong
      about the constraint, not that a layer is needed.

### 1b: Refusal and honest failure

- [ ] T034 [FR-015] Test: `fetch` is refused, and the spawn counter does not move.
- [ ] T035 [FR-015] Test: no audit record claims a success for a refused request.
- [ ] T036 Implement: `src/git/remote.rs`, the refusal list checked before any
      process starts. Makes T034-T035 pass.
- [ ] T037 [FR-006, FR-021] Test: a command git rejects surfaces a failure carrying
      git's stderr **byte for byte**, and a non-zero exit is always a non-success
      result -- including `diff --quiet`, whose exit 1 passes straight through.
- [ ] T038 [FR-022] Test: with `git` removed from PATH, every operation fails plainly,
      names the reason, and no internal fallback is attempted.
- [ ] T039 [FR-022] Test: a git version that differs from the validated one is
      reported as a version mismatch, distinctly from a behavioral failure.
- [ ] T040 Implement: `src/git/version.rs` and `src/error.rs`, the single error type
      converted once at the edge (Article VIII). Makes T037-T039 pass.

### 1c: Byte fidelity and cancellation

- [ ] T041 [FR-018] Test: raw bytes survive the round trip unaltered, including a
      deliberately invalid UTF-8 payload.
- [ ] T042 [FR-018] Test: a display string derived from invalid UTF-8 is marked
      `lossy: true`. Lossiness is a property of presentation, never of the record.
- [ ] T043 Implement: the byte-preserving result and the derived display string.
      Makes T041-T042 pass.
- [ ] T044 [FR-019] Test: cancelling an in-flight operation leaves the repository in
      exactly the state git left it in.
- [ ] T045 [FR-019] Test: cancelling does **not** repair or roll back that state, and
      the re-probe reports it honestly instead. This is the Article I boundary: repair
      would be a private model of git.
- [ ] T046 Implement: cancellation -- terminate the child, re-probe, mark the record
      cancelled. Makes T044-T045 pass.

### 1d: Concurrency

- [ ] T047 [FR-020] Test: two operations against one repository serialize, and the
      second reports that it waited.
- [ ] T048 [FR-020] Test: two operations against **different** repositories run
      concurrently.
- [ ] T049 [FR-020] Test: two serialized operations complete in order and do not
      deadlock. Guards risk R3.
- [ ] T050 Implement: `src/git/lock.rs`, a per-canonicalized-path mutex held by a
      leaf function that only spawns and collects. Makes T047-T049 pass.

### 1e: The audit record

> The corrected requirement. Phase 1e writes a file on every invocation, which is
> why FR-028 exists and why T073 measures it.

- [ ] T051 [FR-004] Test: exactly one record per invocation, with all four fields.
- [ ] T052 [FR-004, FR-005] Test: N invocations produce exactly N records, in issue
      order, none zero and none doubled.
- [ ] T053 [FR-004] Test: the git directory is resolved by asking git, not by
      assuming `.git`. A worktree fixture and a `--separate-git-dir` fixture both
      resolve to their real git dir. This is the test that keeps the tool from
      reimplementing path knowledge git already owns.
- [ ] T054 [FR-004] Test: a record written by one process is readable by a **second,
      separate** process. This is the test that Story 2's "after the fact" actually
      has a mechanism. Expected to fail on first run.
- [ ] T055 [FR-005] Test: two processes appending concurrently produce lines that are
      each individually parseable. A torn or interleaved line is the failure.
- [ ] T056 [FR-016] Test: the record type cannot hold repository state -- a
      compile-fail assertion that the struct has no OID, ref-name, or file-state
      field. This is the structural guard on Article II, and it is what makes
      persistence safe rather than a state tracker.
- [ ] T057 [FR-017] Test: every rendered audit output is labelled as this tool's
      invocations and never as repository history.
- [ ] T058 [FR-027] Test: the log is bounded -- crossing the documented cap applies
      the documented policy.
- [ ] T059 [FR-027] Test: truncation touches **only** the tool's own log file. Assert
      the surrounding git directory is byte-identical before and after, and that no
      other path in the repository changed. Guards the worst plausible accident in
      this feature.
- [ ] T060 [FR-027] Test: a truncation is reported to the caller, never silent.
- [ ] T061 Implement: `src/git/audit.rs` -- the append-only log in the git dir, the
      git-dir resolution, the size cap, the retention policy, and the
      report-on-truncate. Makes T051-T060 pass. If T056 forces a field change, stop
      and re-read the constitution -- that constraint outranks convenience.

### 1f: The differential matrix

- [ ] T062 [FR-007, SC-001] Test: the full matrix -- six fixtures, each with one
      read-only, one mutating, and one failing operation, every comparison equal.
      (`tests/differential/<fixture>.rs`, one file per fixture so they can run in
      parallel.)
- [ ] T063 [FR-008] Test: state observables compare exactly; stdout and stderr
      compare on content with the fixture path normalized. Two fixtures at different
      paths must compare equal.
- [ ] T064 [FR-009] Test: a mutation applied to only one side is caught and the
      differing observable is named.
- [ ] T065 [NFR-006] Run the whole matrix; the invariant *is* the suite. No separate
      assertion.

**Gate**: T062-T064 pass with 0 differences across every observable on all six
fixtures. Core tests green.

---

## Phase 2: Headless Surface

> Article IV: every capability reachable with no window before any GUI exists.
> Feature 001 has no GUI at all, so this phase completes the feature.

- [ ] T066 [FR-013] Test: the CLI reaches every capability in this feature -- `git`,
      `audit`, `diff`, `fixtures`, `version` -- with no display attached.
- [ ] T067 [FR-014] Test: exit 0 on success, non-zero on failure, and git's own exit
      code present in the output even when the process code is ours.
- [ ] T068 [FR-012] Test: `gitflowfy version` prints the tool version and the git
      version found.
- [ ] T069 [FR-004, FR-017, Story 2] Test: `gitflowfy audit` run as a **fresh
      process** after an earlier `gitflowfy git` process exits prints the earlier
      invocation with all four fields and the scope label. This is the end-to-end
      proof that the in-memory design has been genuinely fixed rather than patched.
- [ ] T070 Implement: `src/cli/`, argv parsing, `--json`, exit codes, the single
      error conversion. Makes T066-T069 pass.
- [ ] T071 [NFR-004] Test: run the entire CLI suite with `DISPLAY` empty and
      `SESSIONNAME` unset. Nothing may require a graphical session.
- [ ] T072 [NFR-005] Test: the build produces no UI crate and links no windowing
      toolkit. Asserted by inspecting the workspace members.

**Gate**: every capability reachable with no window. CLI suite green.

---

## Phases 3 and 4: Not Applicable to This Feature

**Phase 3 (graph and render)** and **Phase 4 (UI)** are empty here by design. The
spec excludes the graph, the renderer, and every form of GUI. Listing tasks against
them would be inventing scope.

They begin at feature 002, where `plan.md` already commits to re-running
`node --experimental-websocket spike/run-spike.mjs` against the Article VI budgets and
porting `spike/build-graph.mjs` into the core, per `research.md`.

---

## Phase 5: Integration

- [ ] T073 [NFR-002] Test: measured per-invocation overhead is at or under 5 ms
      median and 25 ms p99 over 100 or more invocations, excluding git's runtime.
      Fails the build on a median breach. **If this fails, NFR-002 is revised with
      the measurement attached and explicit approval -- it is never quietly
      raised.**
- [ ] T074 [FR-028] Test: the overhead measurement is taken with the audit write
      **in place**, not stubbed. A design that measures 5 ms with the write disabled
      has not measured the thing the budget covers. This test exists because the
      cheapest way to pass T073 would otherwise be to switch the write off.
- [ ] T075 [NFR-003] CI: the full suite runs on windows-latest and ubuntu-latest,
      and both are required. The run records git version and platform.
- [ ] T076 [NFR-003, FR-012] Test: the run records the platform field, so a
      single-platform pass cannot masquerade as cross-platform.
- [ ] T077 [Article I, V, VI, VII, VIII, IX] Constitutional audit against all nine
      articles, every exception matched to a row in `plan.md`'s Complexity Tracking.
- [ ] T078 Confirm the deferred Article V states and the full recovery affordances
      are carried into the repository-state feature's scope, so nothing was dropped
      silently.

**Gate**: full suite green on Windows and Linux. Constitutional audit clean.

---

## Tasks with no implementation task

Five tests have no paired implementation. They are **verification tasks**: they
assert a property of something already built, and a failure means a defect upstream
rather than missing code. Leaving them unpaired would look like a gap in the list, so
they are named here.

| Task | Verifies | On failure |
| --- | --- | --- |
| T071 | the CLI suite runs with no display | a dependency is reaching for a window; fix the module, not the test |
| T072 | the build contains no UI crate or windowing toolkit | a dependency crept in -- this is an Article VII breach |
| T073 | the SC-010 overhead budget | **not a code task.** A breach means NFR-002 is revised with the measurement attached and explicit approval. The budget is never quietly raised to fit a result. |
| T074 | the overhead measurement is taken with the audit write enabled | the measurement was taken with the write stubbed out, so the number does not mean what the budget says it means. Fix the measurement, not the budget. |
| T076 | the platform field appears in every run record | T002's preamble is incomplete |

T002 and T065 are also unpaired for different reasons: T002's run preamble is
delivered by T001, the test-only git runner, which is created rather than
implemented-and-tested; T065 is the whole matrix being *run*, which is the gate
itself.

---

## Dependencies

| Task | Depends on | Parallel with |
| --- | --- | --- |
| T003 | T001 | - |
| T004 | T003 | - |
| T005-T011 | T001 | each other (separate files) |
| T012 | T005-T011 | - |
| T013-T017 | T001 | each other (separate files) |
| T018 | T013-T017 | - |
| T019 | T018 | - |
| T020 | T019 | - |
| T021 | T020 | - |
| T022 | T021 | - |
| T023 | T001, T022 | T024, T025, T026, T027 |
| T024 | T001 | T023, T025, T026, T027 |
| T025-T027 | - | each other, and T023, T024 |
| T028 | **G3 + harness pass** | - |
| T029 | T028 | - |
| T030-T032 | T029 | each other |
| T034 | T029 | T030, T031, T032 |
| T037 | T029, T036 | - |
| T041 | T029 | T037, T044, T047, T051 |
| T051 | T029 | T041, T044, T047 |
| T062 | T029, T022 | - |
| T066 | T062 | - |
| T070 | T066-T069 | - |
| T073 | T061, T070 | - |
| T074 | T061 | T075 |
| T075 | T073 | T076 |

## Parallel Execution

**Wave 1** (after T004): T005, T006, T007, T008, T009, T010, T011 -- seven fixture tests,
separate files
**Wave 2**: T013, T014, T015, T016, T017 -- five observable tests, five separate files
**Wave 3**: T023, T024, T025, T026, T027 -- the compliance matrix and the three CI
scripts; T025-T027 touch different files and share nothing with the tests
**Wave 4**: T030, T031, T032, T034 -- command-module constraints and the refusal
**Wave 5**: T041, T044, T047, T051 -- byte fidelity, cancellation, concurrency, audit.
T041, T047 and T051 are **not** strictly parallel: T047 needs the spawn counter
T041's area introduces, and T051 needs the git-dir resolution from T053's
neighbourhood. Treat Wave 5 as T041 -> (T044, T047, T051).

**The rule that produced these waves**: two tasks are parallel only if they touch no
shared file. That is why the seven fixture builders and the five observables are one
file each. It is a real constraint on how the module tree is laid out, and it was
worth designing the tree around rather than faking parallelism in the task list.

---

## Progress Log

| Date | Task | Outcome | Notes |
| --- | --- | --- | --- |
| 2026-09-29 | -- | plan decomposed into 78 tasks | Two deviations from the agent's defaults plus one review correction, all resolved in the spec's favor and recorded above |

---

## Checklist

- [x] Every task traces to an FR, an NFR, or a named constitution article
- [x] Every test task precedes its implementation task
- [x] Phase 0 safety net passes before feature work
- [x] Every in-scope Article V state has a fixture and a test; the deferred ones
      have the narrow obligation tested (T023), with the full affordances explicitly
      carried forward (T078)
- [x] Headless surface done before UI -- this feature has no UI
- [x] Perf benchmark not required: the render path is untouched. NFR-001 asserts that
      as a checked statement (T027), and Phase 5 measures the one number this feature
      is actually accountable for (T073)
- [x] The audit write is inside the measured path (T074), not outside the budget
- [x] Parallel waves contain no shared-file dependencies
- [x] Pure ASCII output