---
description: Feature 001 - a headless surface that executes real git commands, records every invocation, and proves via differential testing that its results are identical to running git directly.
---

# Feature Specification: Headless Git Operations

**Feature Branch**: `001-headless-git-operations`
**Status**: Approved - 14/14 clarifications resolved by the project owner
(2026-09-29). Amended during constitutional review, same date. Constitution
referenced at v1.1.0.
**Created**: 2026-09-29
**Constitution**: `.claude/constitution.md` v1.1.0

### Amendment log for this spec

| Version | Date | Change | Reason |
| --- | --- | --- | --- |
| 1.0.0 | 2026-09-29 | Initial specification, 26 FR + 6 NFR | Feature inception |
| 1.1.0 | 2026-09-29 | Q5 rewritten from in-memory to persisted; FR-004, FR-005, FR-017, FR-027 and FR-028 added or extended; stale constitution reference corrected from v1.0.0 to v1.1.0 | Constitutional review found a self-contradiction. Story 2 is titled "auditable after the fact" and its independent test requires reading the record after a sequence completes, while Q5 placed the log in memory. An in-memory log belongs to the process that wrote it, so the standalone `gitflowfy audit` subcommand the plan exposed would have returned an empty list on every run, and the requirement was unfalsifiable. The requirement was wrong, so the spec changed rather than the plan being rationalized. Persistence does not breach Article II: the record carries four invocation fields and no repository state. |

## Purpose

Every Git client that reimplements Git's operation layer - GitKraken, Fork, and
Git Extensions all do - inherits a permanent constraint: its UI can only expose
what its own private model of the repository understands. That is why each of
them ships a subset of Git's commands, why their undo only covers what they did
themselves, and why they fail or misreport on repository states their model does
not anticipate. GitKraken bundles its own Git build; Fork ships an internal Git
instance it upgrades on its own schedule; Extensions carries an embedded engine.
Each therefore drifts from the Git the user actually has on disk.

This feature builds the alternative: a headless, scriptable surface that runs
the user's real `git` binary, returns structured results, records every
invocation verbatim, and - the part that matters - carries a differential test
harness that proves our surface and raw `git` leave a repository in byte-for-byte
identical states. The harness is the evidence for the product thesis. Without it,
"we shell out to git" is a claim; with it, it is a measured, falsifiable
guarantee that a CI run can enforce forever.

This is the foundation every later feature stands on: the graph, the reflog-based
time travel, and the eventual GUI all consume this surface and none of them may
mutate a repository any other way.

## Why Now

Nothing else can be specified honestly until this exists. Article IV requires
every capability to be reachable headlessly first, and every later feature's
acceptance test ultimately reduces to "the resulting repository state is what git
produces". Without a tested operation surface, each subsequent spec would have to
re-derive that guarantee from prose. The performance spike has already been
validated against a 600k-commit repository, so the tool's speed is no longer the
open question; correctness of the operation layer is.

## Scope

### In Scope

- Executing a Git command as a separate process against a named repository path.
  The supported set is whatever the installed Git documents, with no private
  allowlist (FR-025).
- Capturing, for every invocation: exit code, standard output, standard error,
  and wall-clock duration.
- Recording every invocation verbatim - working directory, argument vector, exit
  code, duration - so that any operation is auditable after the fact.
- Returning a structured result to the caller on both success and failure, with
  Git's own error text surfaced and never swallowed.
- A differential test harness that performs the same operation twice from
  identical starting repository states - once through our surface, once through
  raw Git - and compares the resulting repository state.
- Fixture repository builders for six baseline shapes: linear history, merged
  history, octopus merge, orphan branch, detached HEAD, and empty repository.
- A command-line entry point that reaches every capability above with no window
  open, suitable for use from a script and from CI.

### Out of Scope

- The commit graph, the renderer, and anything that draws a repository. This
  feature produces no visual output of any kind.
- Any graphical interface. Not a stub, not a placeholder - none.
- Time travel, undo, or redo. The reflog is not read by this feature, and no
  "restore to state T" primitive is built here.
- The full Article V state matrix. Only the five baseline fixture shapes are
  built; the remaining six states (rebase in progress, merge conflict,
  cherry-pick in progress, revert in progress, bisect in progress, partial clone,
  sparse checkout, reftable backend, submodule conflicts, corrupt or missing
  reflog) are a later feature. See the Edge Cases table for what this feature
  nevertheless guarantees about them.
- Submodules, Git LFS, and worktrees.
- Credential handling, remotes, and all network operations. `fetch` and `push`
  are not offered. If a caller requests one, the request must be refused
  explicitly rather than attempted and half-completed.
- Repository discovery, recent-repositories lists, or opening a repository from
  a file browser.
- Diff rendering, blame, staging UX, and commit-message composition.
- Naming the command-line executable and the shape of its subcommands. That is
  a design decision for `/plan`, not a product behavior fixed here.

---

## User Stories

> Each story is independently valuable. Priority: P1 (must ship), P2 (should),
> P3 (nice to have). Maximum 5 stories per feature.

### Story 1 - Run a real Git command and get an honest answer (P1)

**As a** developer building tooling on top of Gitflowfy,
**I want** to ask the tool to run a Git command against a repository and get
back the exit code, output, error text, and duration,
**so that** I never have to guess what Git did or why it refused.

**Independent test**: Point the surface at a freshly built linear-history fixture,
run `rev-parse HEAD`, and assert the returned exit code, stdout bytes, and
duration are present and match a direct invocation of the same command in the
same directory.

**Acceptance scenarios**:

1. **Given** a repository path containing a linear history, **when** a caller
   requests a documented read-only Git command, **then** the result contains a
   zero exit code, the command's standard output unchanged, empty standard
   error, and a duration greater than zero; and the invocation appears in the
   audit record with its working directory, argument vector, exit code, and
   duration.
2. **Given** a repository path containing a linear history, **when** a caller
   requests a documented mutating Git command such as `commit`, **then** the
   repository's resulting ref OIDs and the output of `status --porcelain` are
   identical to the output of the same command run directly by `git` in an
   otherwise identical repository.
3. **Given** a request that Git rejects, **when** the command exits non-zero,
   **then** the caller receives a failure result that still carries the exit
   code, the standard error text exactly as Git wrote it, and the standard
   output; no error text is discarded, rewritten, or replaced with a generic
   message.
4. **Given** a path that is not a Git repository, **when** a command is
   requested, **then** the caller receives a failure result naming the path,
   and the process does not hang, crash, or attempt recovery.
5. **Given** a path that is not a Git repository, **when** a command is
   requested, **then** no entry claiming success is written to the audit
   record; the attempted invocation is still recorded with its exit code.

---

### Story 2 - Every operation is auditable after the fact (P1)

**As a** user whose repository was changed by something,
**I want** to see exactly which Git invocations the tool performed, with what
arguments, where, and how they ended,
**so that** I can trust what happened and reconstruct it if something went wrong.

**Independent test**: Perform a known sequence of Git operations through the
surface in a fixture, then read the audit record and assert that the recorded
invocations equal the requested ones, one-for-one, in order, with complete
fields.

**Acceptance scenarios**:

1. **Given** a successful command, **when** the invocation completes, **then**
   exactly one audit record exists for it, containing the working directory, the
   argument vector as issued, the exit code, and the duration.
2. **Given** a failed command, **when** the invocation completes, **then**
   exactly one audit record exists for it, with the same four fields, and its
   exit code matches the code reported to the caller.
3. **Given** a sequence of N invocations, **when** the sequence finishes,
   **then** exactly N records exist, in invocation order, and no invocation
   produced zero records or two records.
4. **Given** an audit record for a mutating command, **when** the record is
   read, **then** the argument vector is sufficient to reproduce the mutation by
   hand in a terminal, and the working directory is the directory the mutation
   was applied to.
5. **Given** a repository that was also modified by an operation performed
   outside the tool, **when** the audit record is read, **then** it contains no
   entry for that external operation, and it does not claim to describe the
   repository's complete history.

---

### Story 3 - Prove our results are Git's results (P1)

**As a** maintainer of this codebase,
**I want** every operation to be executed twice from identical starting states -
once through our surface, once through raw `git` - and the resulting repository
states compared,
**so that** a claim of Git equivalence is enforced by a failing test rather than
trusted.

**Independent test**: Run the differential harness over the six baseline
fixtures with at least one read-only, one mutating, and one failing operation
each; the harness passes only if all comparisons are equal.

**Acceptance scenarios**:

1. **Given** two repositories built to the same specification, **when** the same
   operation is performed through the surface in one and directly by `git` in
   the other, **then** the harness compares the resulting refs, the resolved
   `HEAD` state, the index state, the working tree contents, and the reflog
   entries, and reports equal on all of them.
2. **Given** a difference in any compared observable, **when** the comparison
   runs, **then** the harness reports the specific observable that differs and
   fails; it does not report a generic mismatch.
3. **Given** the six baseline fixtures - linear history, merged history, octopus
   merge, orphan branch, detached HEAD, empty repository - **when** the harness
   runs, **then** every fixture has been exercised with at least one
   read-only, one mutating, and one failing operation, and every comparison is
   equal, or the run fails.
4. **Given** a command that fails in Git, **when** the differential run
   executes it on both sides, **then** both sides produce the same exit code and
   the same resulting repository state, and a difference in failure behavior is
   reported as a failure of this feature.
5. **Given** a differential test run, **when** it completes, **then** the
   output records the version of the Git binary used, on which platform, so a
   later failure on a different Git version can be distinguished from a
   regression.

---

### Story 4 - Real repositories, not mocks, ready for later features (P2)

**As a** developer writing the next feature,
**I want** to obtain a genuine repository in a known state - linear, merged,
octopus, orphan branch, detached HEAD, empty - built by real Git,
**so that** I can write tests against real repository states instead of
invented ones.

**Independent test**: Build each of the five fixtures and assert that the shape
the fixture claims to have is actually present, verified by inspecting the
built repository with Git itself.

**Acceptance scenarios**:

1. **Given** the linear-history fixture, **when** it is built, **then** the
   repository has a single branch whose history contains no merge commits, and
   every commit has exactly one parent.
2. **Given** the merged-history fixture, **when** it is built, **then** the
   repository contains a merge commit with exactly two parents, and both parents
   are reachable.
3. **Given** the octopus-merge fixture, **when** it is built, **then** the
   repository contains a merge commit with three or more parents, and every one
   of those parents is reachable from it.
4. **Given** the orphan-branch fixture, **when** it is built, **then** the
   orphan branch shares no history with the branch it was created from, proven
   by the merge base between them being absent.
5. **Given** the detached-HEAD fixture, **when** it is built, **then** `HEAD` is
   not a symbolic reference to any branch, and resolves to an existing commit.
6. **Given** the empty-repository fixture, **when** it is built, **then** `HEAD`
   names a branch that does not yet exist, and no commit is reachable.
7. **Given** the same fixture specification, **when** it is built twice in
   different locations, **then** the resulting commit OIDs are identical, so
   that a comparison against a Git-produced repository is meaningful.

---

### Story 5 - Drive it from a script or a CI job (P2)

**As a** user who wants to automate repository work in CI,
**I want** every capability of this feature reachable from a command line with
no window open, and a meaningful exit code to branch on,
**so that** I can use Gitflowfy in a pipeline without a desktop session.

**Independent test**: Run each capability from a shell script with no display
available and assert the process exits with the documented code and the expected
output.

**Acceptance scenarios**:

1. **Given** a repository path and a Git command, **when** it is issued through
   the command-line entry point in an environment with no display, **then** the
   command runs to completion, prints the result to standard output, and exits
   with a code that distinguishes success from failure.
2. **Given** a command that fails, **when** it is run from the command-line
   entry point, **then** the process exit code is non-zero and Git's error text
   appears in the process output; nothing is hidden in a window that never
   appeared.
3. **Given** the full set of capabilities delivered by this feature, **when** the
   command-line entry point is inspected, **then** every one of them is
   reachable from it, and none requires a graphical session.
4. **Given** a machine where the Git binary is absent or unusable, **when** any
   command is issued, **then** the entry point reports plainly that Git could
   not be used, names the reason, and does not attempt a fallback
   implementation.

---

## Requirements

### Functional

- **FR-001**: The system SHALL execute every repository operation by running the
  real `git` binary as a separate process. It SHALL NOT reimplement any part of
  Git's operation layer.
- **FR-002**: The system SHALL accept the operations it supports as a
  documented Git invocation with an explicit working directory, and SHALL NOT
  accept a free-form command string to be interpreted.
- **FR-003**: The system SHALL return, for every invocation, the exit code, the
  standard output, the standard error, and the elapsed duration.
- **FR-004**: The system SHALL record every invocation, successful or not, with
  its working directory, its argument vector, its exit code, and its duration.
  The record SHALL be durably appended to a log the tool owns, inside the
  repository's own Git directory, so that a later, separate process can read
  what this tool did. The location SHALL be resolved by asking Git for its Git
  directory rather than by the tool assuming a `.git` path, so that worktrees
  and separate Git directories resolve correctly.
- **FR-005**: The system SHALL record invocations in the order they were issued.
  Records SHALL be append-only, one record per line, and a record from a
  concurrent process SHALL NOT interleave with or corrupt another.
- **FR-006**: The system SHALL surface Git's failure output to the caller. A
  non-zero exit SHALL never be reported as success, and no failure SHALL be
  converted into a generic message that discards Git's own text.
- **FR-007**: The system SHALL provide a differential harness that applies the
  same operation through its own surface and directly through `git`, from two
  repositories in identical starting states, and compares the results. The
  matrix covers, for each baseline fixture, at least one read-only operation, one
  mutating operation, and one operation that fails - the three behaviors the
  surface must get right. The matrix is extensible as later features add
  operations.
- **FR-008**: The differential comparison SHALL cover, at minimum: all local
  refs and their OIDs, the resolved `HEAD` state including whether it is
  attached or detached, the index state, the working tree contents, and the
  reflog entries produced. These state observables are compared **exactly**.
  Standard output and standard error are compared on content, normalizing only
  the fixture's own repository path and Git's own nondeterministic output such as
  timings; they are not required to be byte-identical, because the two fixtures
  necessarily live at different paths and that difference is an artifact of the
  test rather than a behavioral divergence.
- **FR-009**: A difference in any compared observable SHALL cause the harness to
  fail and SHALL identify which observable differed.
- **FR-010**: The system SHALL provide builders that create real repositories,
  produced by real Git, in the following states: linear history, merged history,
  octopus merge, orphan branch, detached HEAD.
- **FR-011**: Fixtures built from the same specification SHALL produce the same
  commit OIDs, so that a fixture can be compared against a Git-produced
  repository. The pinning policy is: author and committer name and email, author
  and committer date, time zone, locale, and line-ending behavior are all pinned
  to fixed values by the fixture builder, and the builder is permitted to set
  repository configuration and the process environment to do so. The pinned
  values are recorded in the test output so a failure can be attributed.
  Signing is disabled for fixture repositories, since a signing prompt or an
  absent key would make the fixture non-deterministic.
- **FR-012**: The system SHALL record the Git binary's version and the platform
  in the output of every test run.
- **FR-013**: The system SHALL provide a command-line entry point through which
  every capability in this feature is reachable without a graphical session.
- **FR-014**: The command-line entry point SHALL exit 0 on success. On failure
  it SHALL exit with a non-zero code that the caller can branch on, and it SHALL
  preserve Git's own exit code in its output so no information is lost. The
  process exit code is a summary; Git's code is always available verbatim.
- **FR-015**: The system SHALL refuse, explicitly and without side effects, any
  request for a remote or network operation. It SHALL NOT partially perform one.
- **FR-016**: The system SHALL not maintain any record of repository state for
  the purpose of undo or history. The audit record is a log of invocations
  performed, not a model of the repository, and SHALL NOT be usable as a
  substitute for Git's reflog.
- **FR-017**: The audit record SHALL NOT be presented as a complete account of
  everything that has happened to a repository. It is scoped to invocations
  performed by this tool, as recorded in FR-004, and is labeled as such wherever
  it is presented. Its scope is bounded by **the invocations this tool performed**,
  and it never claims to be a substitute for Git's reflog. Because the record
  names only the tool's own invocations, work done in a terminal, in another IDE,
  or in a colleague's clone is absent from it **by design and is not a defect**;
  that absence is the reason the reflog exists.
- **FR-018**: The system SHALL preserve the bytes Git wrote to standard output
  and standard error, without re-encoding, truncation, or normalization. The
  returned result carries the raw bytes, which are the authoritative form. A
  caller MAY additionally request a display string derived from those bytes for
  human presentation; that derived string is permitted to be lossy where the
  bytes are not valid text in the assumed encoding, and when it is, the result
  MUST indicate that the display string is a lossy rendering rather than Git's
  verbatim output. Lossiness is a property of the presentation, never of the
  stored or audited result.
- **FR-019**: A caller SHALL be able to cancel an operation that is in flight.
  Cancellation SHALL terminate the running Git process. Because an interrupted
  Git process can leave a repository mid-operation, the system SHALL NOT attempt
  to repair or roll back that state - that would be a private model of Git
  behavior, forbidden by Article I. Instead the system SHALL re-probe the
  repository and report the state Git actually left behind, and the audit record
  SHALL mark the invocation as cancelled. An operation cancelled mid-mutation
  MUST leave the repository in exactly the state Git left it in, and the tool
  MUST NOT clean that state up on the user's behalf without being asked.
- **FR-020**: The behavior when two operations are issued against the same
  repository at the same time SHALL be defined, not undefined. Operations
  targeting the same repository are serialized: the second waits for the first
  to complete rather than running concurrently, because Git's own index and ref
  locking does not support two mutating processes in one repository and the
  resulting failure would be a Git lock error the user cannot act on. Operations
  targeting *different* repositories MAY run concurrently. A serialized
  operation reports that it waited.
- **FR-021**: The system SHALL treat a non-zero exit as a signal to be reported,
  not as a verdict on success. A non-zero exit always surfaces as a non-success
  result carrying Git's own exit code, stdout, and stderr. The system SHALL NOT
  reinterpret a non-zero exit as a normal result for any command - including
  commands such as `diff --quiet`, where a non-zero exit is meaningful to a
  caller. Interpreting exit-code meaning per command would require the tool to
  know each command's semantics, which is the private model of Git that
  Constitution Article I forbids. The caller decides what a non-zero exit means
  for its own purpose, using the exit code the tool reports faithfully.
- **FR-022**: The system SHALL NOT silently fall back to any internal
  implementation of Git behavior when the real `git` binary cannot be used. The
  system SHALL verify at startup that a usable `git` binary is present and
  reports a version, and SHALL record that version. If the binary is missing,
  not executable, not on the search path, or reports no parseable version, the
  system SHALL report that fact plainly, name the reason, and refuse to perform
  operations. It SHALL NOT fall back to an internal implementation, and it
  SHALL NOT accept a different Git implementation without saying so. This
  feature sets no minimum Git version; it records the version found and
  reports a mismatch against the version the test suite was validated against.
- **FR-023**: The system SHALL run against real repositories produced by real
  Git. Mocking Git's behavior is permitted only to force a failure that Git
  cannot naturally produce, and the differential harness SHALL NOT use mocks on
  either side of a comparison.
- **FR-024**: The system SHALL not depend on any Git object model of its own.
  Every fact about a repository SHALL be obtained by asking Git.
- **FR-025**: The system SHALL accept any Git command in Git's own documented
  command set, determined by asking the installed Git what it supports rather
  than by a private allowlist. The system SHALL NOT restrict commands for
  reasons of its own convenience. It SHALL refuse only the operations excluded
  by this feature's scope - remote and network operations - and a refusal
  SHALL be explicit and side-effect-free.
- **FR-026**: The system SHALL treat a repository with no commits yet, where
  `HEAD` points at a branch that does not exist, as a normal state. An empty
  repository is within this feature's baseline: it is a fixture the differential
  harness builds and compares, and operations against it return Git's own exit
  code and output.

- **FR-027**: The audit log SHALL be bounded. When it exceeds a documented size
  cap, the tool SHALL truncate it to a documented retention policy rather than
  grow without limit. The tool SHALL remove or rewrite **only** the log file it
  owns, and SHALL NOT delete, truncate, or modify any other path in the user's
  repository. The cap and the policy SHALL be documented and SHALL be reported
  when a truncation occurs, never applied silently.
- **FR-028**: Writing the audit record is part of the per-invocation cost, and
  SHALL therefore be included in the overhead measured against SC-010. A design
  that meets the budget without the write but misses it with the write does not
  meet the budget.


### Non-Functional

<!-- Pull thresholds from Constitution Article VI where they apply. Do not
     invent numbers; cite the article. -->

- **NFR-001**: The performance budgets of Constitution Article VI govern the
  extract, graph-size, load-into-renderer, and frame-time paths. This feature
  touches none of them and therefore does not set or claim a number for them.
  Those budgets are measured by the same headless CI harness when the graph
  feature lands, and a regression to a non-typed-array graph model fails the
  build.
- **NFR-002**: The overhead this surface adds on top of the Git process it runs
  SHALL be measured by the differential harness for every invocation, excluding
  Git's own runtime, and reported separately per platform. The budget is:
  **median overhead <= 5 ms, p99 overhead <= 25 ms, measured over at least 100
  invocations per platform**, and the harness fails the build if the median
  exceeds the budget. The median is the budgeted number because a single
  invocation is dominated by process startup, which is Git's cost and not ours;
  the p99 guards against our layer adding a pathological path. First
  measurement of the real number may propose a revision of this budget, with the
  measurement attached, and requires approval - the budget is not silently
  raised to fit a result.
- **NFR-003**: The differential harness SHALL run in CI on at least Windows and
  Linux, per Constitution Article IX, and its results SHALL be published with
  the Git version used.
- **NFR-004**: The command-line entry point SHALL be usable from a
  non-interactive session with no display attached, per Constitution Article IV.
- **NFR-005**: Every capability in this feature SHALL be reachable headlessly
  before any graphical presentation of it is designed. This feature delivers
  only the headless half.
- **NFR-006**: No operation performed by this feature SHALL leave a repository
  in a state that Git itself would not have produced, per Constitution Article
  I. This is the invariant the differential harness exists to test.

### Constitutional Constraints

<!-- Which articles bind this feature, and what do they forbid here?
     This is not boilerplate - it prevents the planner from proposing
     something that Article I or IV rules out. -->

- **Article I** - This feature is the direct implementation of this article, and
  it forbids the most tempting shortcut available here. No embedded Git binding
  may appear in the dependency tree, and no Git object model may exist
  internally. The audit record required by Article I is a log of invocations
  performed; it is explicitly not permitted to grow into a model of repository
  state, which is the failure mode Article II prohibits in a different place.
- **Article II** - Forbids a parallel state tracker. Nothing in this feature may
  remember what the repository looked like in order to reason about it later. If
  a fact is needed, ask Git. This is why the audit record is scoped to
  invocations and outcomes, and why no undo primitive appears in this feature
  even though the reflog would make one easy to bolt on.
- **Article III** - Test-first is binding on this feature more than on any
  other, because the tests here are the product. The differential harness
  scenarios are the acceptance criteria; they must be written, observed failing,
  approved, and only then implemented.
- **Article IV** - Forbids any graphical surface and any capability that exists
  only behind a window. Story 5 is this article made testable. It also forbids
  the process boundary from being introduced prematurely: the interface is
  separated first, and this spec says nothing about how the pieces are deployed.
- **Article V** - Forbids crashing, hanging, or silently corrupting on any
  unusual repository state, including the six states this feature does not yet
  model. For those six the binding obligation is narrow but real: an operation
  against such a repository must return Git's own exit code and output, must be
  recorded, and must not hang. Detached HEAD is the one Article V state this
  feature fully covers, because it is one of the five baseline fixtures. The
  remaining Article V states are not in scope, and this feature must not pretend
  to handle them.
- **Article VI** - Sets budgets on paths this feature does not touch, and
  therefore requires the harness this feature builds to be the one later used to
  measure them. It also forbids this feature from setting its own unmeasured
  performance numbers; NFR-002 measures overhead rather than asserting a
  threshold that has not been agreed.
- **Article VII** - Forbids adding components or build targets for this feature.
  The number the plan may propose is fixed by the article itself, and any
  addition requires a plan amendment. The article's example component names are
  the constitution's vocabulary, not a decision this spec makes.
- **Article VIII** - Forbids a wrapper whose only job is to reshape an existing
  API. This binds the shape of the operation surface directly: the surface
  exists to satisfy Article I and Article IV, and a layer that only re-exports
  or re-types Git's own interface is forbidden. Error handling is one
  documented set of outcomes at the core boundary, converted once at the edge.
- **Article IX** - Forbids mocking Git. Every scenario in this feature runs
  against a real repository built by real Git, the version is recorded, and CI
  runs on at least Windows and Linux.

### Edge Cases

<!-- Constitution Article V requires unusual repo states to be handled.
     List the states this feature touches. -->

The full Article V matrix of eleven states is a later feature. This feature
covers five of them outright, and for the remaining six it guarantees only that
it does not hang, crash, corrupt, or lie.

| Case | This feature's obligation | Status |
| --- | --- | --- |
| Detached HEAD | Fixture built and differential-compared; operations on a detached `HEAD` return Git's exit code and output and leave Git's state untouched. | Covered |
| Merged history | Fixture built and differential-compared; the merge commit's parent count and reachability are verified. | Covered |
| Octopus merge | Fixture built and differential-compared; a merge commit with three or more parents is verified to survive the surface unchanged. | Covered |
| Orphan branch | Fixture built and differential-compared; the absence of a shared merge base is verified. | Covered |
| Linear history | Fixture built and differential-compared; the baseline against which all others are judged. | Covered |
| Rebase in progress | Not modelled. An operation against such a repository returns Git's exit code and stderr, is recorded, and does not hang or alter state beyond what Git does. No recovery affordance. | Deferred |
| Merge conflict (`MERGE_HEAD` present) | As above. No conflict resolution, no display. | Deferred |
| Cherry-pick in progress | As above. | Deferred |
| Revert in progress | As above. | Deferred |
| Bisect in progress | As above. | Deferred |
| Partial clone | As above. No missing-object resolution is attempted by this feature. | Deferred |
| Sparse checkout | As above. | Deferred |
| Reftable backend | As above. | Deferred |
| Submodule conflicts | As above. Submodules are out of scope entirely. | Deferred |
| Corrupt or missing reflog | As above. The harness compares reflog entries, so a missing reflog surfaces as a comparison difference rather than being silently accepted. | Deferred as a user-facing state; the comparison is not |
| Empty repository, no commits yet | Fixture built and differential-compared; an operation against a repository with an unborn `HEAD` returns Git's exit code and output and does not hang. This is a normal state, not an error. | Covered |
| Requested path is not a repository | Failure result naming the path; no hang, no crash, no fallback. | Covered |
| Requested path does not exist | Failure result distinguishing this case from the case above. | Covered |
| Git binary missing, not executable, or an unsupported version | Plain, explicit failure. No internal fallback implementation is ever attempted. | Covered |
| Command exits non-zero | Failure result carrying Git's own exit code, stderr, and stdout. | Covered |
| Command exits non-zero but means a normal result | Always surfaced as a non-success result carrying Git's exit code. The tool does not reinterpret it; the caller decides what the code means. Per FR-021. | Covered |
| Command that would prompt for input | **No private allowlist** - the command set is whatever Git supports. To prevent a hang, a command is issued with input closed or empty, so a prompting command fails fast with Git's own "cannot prompt" behavior instead of blocking. | Covered |
| Command producing output that is not valid UTF-8 | Bytes preserved unchanged and authoritative; a display string may be lossy, and is marked lossy when it is. Per FR-018. | Covered |
| Command producing very large output | No truncation of the bytes returned; whether the full output is retained in memory or streamed is a design decision, not a behavior fixed here. | Design decision deferred |
| Path containing spaces or non-ASCII characters | Treated as an opaque path; the argument vector recorded is exactly what Git received. | Covered |
| Caller cancels an in-flight operation | The Git process is terminated; the repository is left exactly as Git left it; the tool re-probes and reports the resulting state, marked cancelled in the audit record. No repair, no rollback. Per FR-019. | Covered |
| Two operations against one repository concurrently | Serialized; the second waits and reports that it waited. Different repositories run concurrently. Per FR-020. | Covered |
| Remote or network operation requested | Refused explicitly, with no partial effect. | Covered |

---

## Success Criteria

<!-- Must be measurable. "Works well" is not a criterion. -->

- **SC-001**: The differential harness passes for all six baseline fixtures, with
  0 differences across all compared observables, and fails on any difference.
- **SC-002**: Every invocation issued through the surface in a test run has
  exactly one audit record. Verified by an assertion that record count equals
  invocation count; the count of unmatched records is 0.
- **SC-003**: 100% of records contain all four required fields - working
  directory, argument vector, exit code, duration. A record missing any field is
  a test failure.
- **SC-004**: 100% of non-zero exits are surfaced to the caller as a failure
  result carrying Git's own error text. The count of swallowed or generic
  errors is 0.
- **SC-005**: 0 repository mutations are performed without a real `git` process.
  Verified by an automated dependency-tree check for Git implementation
  libraries, per Article I's own enforcement clause, plus manual review that no
  Git object model exists.
- **SC-006**: 100% of the capabilities in this feature are reachable from the
  command-line entry point with no display attached. The count of capabilities
  requiring a window is 0.
- **SC-007**: Every differential test run outputs the Git version and platform
  used. Verified by asserting the run output is non-empty for both fields.
- **SC-008**: The differential harness passes on both Windows and Linux in CI,
  per Article IX, and fails the build on either platform.
- **SC-009**: A fixture built twice from the same specification yields identical
  commit OIDs, 0 differing OIDs, so that comparisons against Git-produced
  repositories are meaningful. Every fixture is built at least twice in a run to
  prove determinism.
- **SC-010**: Measured per-invocation overhead, excluding Git's runtime, is
  **<= 5 ms median and <= 25 ms p99** over >= 100 invocations, on each platform,
  and the harness fails the build when the median exceeds the budget. The report
  states the git version, the platform, and the sample size.
- **SC-011**: For the six baseline fixtures, the difference between our
  surface's result and raw Git's result, measured as mismatches across refs,
  `HEAD` state, index state, working tree contents, and reflog entries, is
  exactly 0 in 100% of cases.

---

## Competitive Differentiation

<!-- Be honest. If a competitor already does this, say how we beat them.
     If we are merely matching them, flag it: that is not a differentiator. -->

| Capability | GitKraken | Fork | Extensions | Our edge |
| --- | --- | --- | --- | --- |
| Headless, scriptable surface | **Has one.** Ships `gk`, a public-preview CLI with a `--output json` flag, git passthrough, and an MCP server. | No headless surface. GUI only; has a user-configurable "custom command" hook. | No headless surface. GUI plus shell/Visual Studio integration. | **Parity on existence, not on commitment.** Having a CLI is not our differentiator any more - see the note below. Our claim narrows to: ours requires no account, no telemetry, and no platform service, and the headless surface is the only surface, not an optional extra. |
| Uses the Git the user already has | **No.** Bundles and upgrades its own Git build. | **No.** Ships an internal Git instance it upgrades on its own schedule. | Bundles its own engine. | **Yes, and it is verifiable.** Our differential harness asserts equivalence against the user's Git on the user's machine. This is a concrete, testable difference, and it is the one the harness exists to prove. |
| Git command coverage | Subset - the `gk` surface plus passthrough | Subset - what the GUI exposes | Subset - what the GUI exposes | Coverage follows from Article I: the constraint is the absence of a private model, not the size of a wrapper. Honest caveat: coverage is only as good as the harness that tests it, so coverage claims are scoped to what the differential matrix covers. |
| Evidence of Git equivalence published as a test | Not published. | Not published. | Not published. | **Real but narrow.** Nobody publishes their equivalence testing, so this is an advantage in method transparency, not a proven superiority over anyone's internals. It becomes a genuine edge only if the harness runs on every commit and its results are published. |
| Records every invocation for the user | No. Its internal history exists to power its own undo and is not a user-facing audit log. | No. | No. | **Real.** A complete, user-inspectable record of what the tool ran, including operations the user did not initiate is something none of the three offer. Note this is not the same as undo, and this feature does not build undo. |
| Undo covering work done outside the tool | Partial - bounded by its internal tracker. | Partial. | Partial. | **Out of scope here.** The reflog-based time travel feature is where this claim is earned. Claiming it now would be a marketing claim with no test behind it. |

**Note on the Article IV rationale.** The constitution originally stated that
"none of the incumbent tools can be used in CI, in a script, or by another
program." As of 2026-09-29 that was no longer accurate for GitKraken, which ships
`gk`. The article's *requirement* - every capability reachable headlessly first -
is unchanged and is enforced by SC-006. **The constitution has been amended to
v1.1.0**; its rationale now claims parity on headless availability and names the
three narrower edges that survive. A spec may no longer claim a headless
advantage without naming which of those three it means.

---

## Open Questions

<!-- Anything the user must decide. Keep these as explicit questions; do not
     guess. -->

All 14 resolved. Decisions were made by the project owner, who delegated them.
Each decision is recorded in the requirement it governs, so a reader of the
requirement sees the reasoning without needing this table.

| # | Question | Resolution | Governs |
| --- | --- | --- | --- |
| Q1 | Failure surfacing: raw passthrough or normalized categories? | **Raw is authoritative.** Exit code, stdout, and stderr pass through unchanged. A normalized category is *derived* alongside, never replaces the raw data. Rejection code is always available. | FR-006, FR-021 |
| Q2 | May an in-flight operation be cancelled, and who owns the leftover state? | **Yes, and Git owns the state.** The process is terminated; the tool does not repair or roll back, because that is a private model of Git behavior. It re-probes and reports, and marks the record cancelled. | FR-019 |
| Q3 | Parallel, serialized, or rejected for one repository? | **Serialized per repository; parallel across repositories.** Git's index and ref locking does not support two mutating processes in one repo, and the resulting lock error is not actionable by the user. | FR-020 |
| Q4 | Non-UTF-8 output: is lossy presentation acceptable? | **Bytes authoritative, presentation may be lossy.** The returned bytes are what Git wrote. A display string is a derived, explicitly-marked convenience. Lossiness never reaches storage or the audit record. | FR-018 |
| Q5 | Does the audit record persist across restarts, and may the user inspect it? | **Yes - persisted, append-only, in a log the tool owns inside the repository's Git directory.** An earlier draft said "in-memory for this feature", which was incompatible with Story 2: an in-memory log belongs to the process that wrote it, so "auditable after the fact" from a separate process would always have returned an empty list. Persistence does not violate Article II, because the record holds the four invocation fields and no repository state - what Article II forbids is a model of the repository, not a log of what this tool ran. The log is bounded by a documented size cap and retention policy, the tool touches only its own file, and a truncation is reported rather than silent. | FR-004, FR-005, FR-017, FR-027, FR-028, Story 2 |
| Q6 | Minimum Git version and behavior when it is missing? | **No minimum is set.** The version found is recorded and compared against the validated version, and a mismatch is reported. Missing, non-executable, or versionless git fails plainly with no fallback. The spec does not invent a floor the product has not been tested against. | FR-022 |
| Q7 | Acceptable per-invocation overhead over raw git? | **<= 5 ms median, <= 25 ms p99**, over >= 100 invocations per platform, excluding git's runtime. Median is budgeted because a single invocation is dominated by Git's own process startup. Revision requires the measurement attached and explicit approval. | NFR-002, SC-010 |
| Q8 | Must stdout/stderr match byte-for-byte in the differential run? | **No - state is byte-exact, output is semantic.** Repository state (refs, OIDs, index, working tree, reflog) is compared exactly. stdout/stderr are compared on content, normalizing only the fixture path and Git's own nondeterministic output. Byte-exact output comparison would fail on the two repos living at different paths, which is a test artifact, not a defect. | FR-007, FR-008, SC-001 |
| Q9 | Which fixture inputs must be pinned for reproducible OIDs? | **Identity, both dates, time zone, locale, and line-ending behavior, with signing disabled.** The builder may set repository configuration and process environment. Pinned values are recorded in test output. | FR-011, SC-009 |
| Q10 | May any documented Git command be restricted? | **No private allowlist.** The set is whatever the installed Git supports. Only the scope exclusions are refused. A prompting command is made non-blocking by closing stdin, not by banning it. | FR-025 |
| Q11 | Which operations must the differential matrix cover? | **At least one per fixture, plus one read-only, one mutating, and one failing operation per fixture.** That yields coverage of success, state change, and error propagation - the three behaviors the surface must get right. The matrix grows as features add operations. | FR-007, SC-001, SC-011 |
| Q12 | Does a non-zero exit always surface as failure? | **Always surfaced as non-success, carrying Git's code. Never reinterpreted.** Deciding that `diff --quiet`'s exit 1 is "normal" requires knowing each command's semantics - the private model Article I forbids. The caller interprets. | FR-021 |
| Q13 | Is an empty repository in this feature's baseline? | **Yes, included.** It is cheap to build, it is the first state every repository is in, and excluding it would leave the most common starting condition untested. | FR-026 |
| Q14 | Amend the Article IV rationale? | **Yes. Constitution amended to v1.1.0.** The false sentence was removed, headless availability is marked parity, and three narrower edges are named. Leaving a false premise in the constitution would let every future spec inherit it. | Article IV, SS4.2 |

---

## Clarification Check

> Spec Kit's failure mode is the model quietly inventing details. Every
> ambiguity MUST be marked, not guessed.

- [x] No implementation details (no tech stack, no file paths, no APIs) - the
      document names behaviors and observable outcomes only. The stack, the
      process model, the component count, and the executable's name are all left
      to `/plan`.
- [x] Every requirement is testable and unambiguous - every functional
      requirement states a decided behavior. 0 `[NEEDS CLARIFICATION]` markers
      remain. All 14 questions are resolved, and each resolution is written into
      the requirement it governs rather than left in a table for the reader to
      join up. No requirement relies on a guessed default.
- [x] Every acceptance scenario has observable outcomes - each `then` clause
      names a specific comparison, a count, an exit code, or an exact output,
      not a state of satisfaction.
- [x] All ambiguities resolved - 14 questions, 18 markers, **0 remaining**. Every
      question was decided by the project owner and its resolution is stated
      inline in the governing requirement. No question was resolved by guessing.
- [x] Success criteria are numeric - SC-001 through SC-011, each with a count, a
      percentage, or a zero-tolerance invariant. SC-010 now carries the agreed
      overhead budget (5 ms median, 25 ms p99) and no criterion remains blocked.
- [x] Out of scope is explicit - nine items listed, including the full Article V
      matrix, the graph, the GUI, and all remote operations.
- [x] Constitutional constraints identified - all nine articles addressed, with
      the specific prohibition stated for each rather than "the article applies".
- [x] Competitive claims are honest about parity vs. advantage - the headless
      surface is marked **parity** because GitKraken ships one; the
      use-the-user's-Git and audit-record rows are marked as real edges; the
      differential-testing row is marked as an advantage in method transparency
      rather than proven superiority; the undo row is marked out of scope
      because this feature does not build it.
