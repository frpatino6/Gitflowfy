# Gitflowfy Constitution

**Version 1.1.0 - Ratified 2026-09-29**

This constitution is binding. Every spec, plan, and implementation is checked
against it. When a change conflicts with it, the constitution wins unless it is
amended first (SS4.2). Complexity that violates an article must be justified in
writing in the plan's Complexity Tracking table, or removed.

---

## Preamble: What we are building

Gitflowfy is a Git client that must be **strictly better** than GitKraken, Git
Fork, and Git Extensions. GitKraken is the floor, not the target. "Better" is
only true if it is measurable - every claim in this constitution maps to a
verifiable test.

The product thesis: **one architectural decision produces nearly every
advantage over the incumbents.** By shelling out to the real `git` binary and
treating git's reflog as the only source of truth, we inherit full command
coverage, cannot desynchronize from the CLI, and can undo work we did not
perform. Reimplementing Git's operation layer - as every incumbent does to
support their own undo - is what forces them to limit the UI to a subset of
commands and to break on unusual repository states.

---

## Article I - Shell Out. Never Reimplement Git.

> All repository mutations MUST be performed by executing the real `git`
> binary. Reimplementing Git's operation layer is forbidden.

Rationale: GitKraken, Fork, and Extensions each maintain a parallel model of
repository state so they can offer their own undo. That is why they expose only
a subset of commands in the UI, and why they fail on states their model does
not anticipate. We get 100% command coverage for free by not writing the code.

Consequences:
- `libgit2`, `jgit`, `gix`, or any embedded Git binding MUST NOT be used for
  mutating operations.
- Every mutating action is a `spawn` of a documented `git` invocation.
- Invocations MUST be recorded verbatim in an audit log with cwd, argv, exit
  code, and duration.
- If a needed operation appears to require embedded Git, that is a signal the
  architecture is wrong - fix the design, not the constraint.

Enforcement: `crates/core/src/git/` contains no Git object model. A CI check
asserts no Git implementation crate appears in the dependency tree.

---

## Article II - The Reflog Is The Only State.

> Undo, redo, history, and time-travel MUST be derived from Git's reflog.
> A parallel state tracker is forbidden.

Rationale: the incumbent tools track their own operations. Undo therefore only
covers what the tool did; work done in a terminal, another IDE, or a colleague's
clone is invisible to it, and the UI desynchronizes. Git already writes a
complete, ordered, durable log of every ref movement. Reading it is strictly
more correct than inventing a second ledger.

Consequences:
- No `OperationLog`, `HistoryTracker`, or equivalent type may exist.
- "Restore the repository to the state at time T" is the primitive. "Undo my
  last action" is a special case of it.
- Undo MUST span operations performed outside the tool.
- If the reflog is missing or corrupt, the app MUST say so plainly and offer
  manual recovery - it MUST NOT silently fall back to a private tracker.

Enforcement: `crates/core/src/state/` MUST NOT contain an operation log type.

---

## Article III - Test First. Non-Negotiable.

> No implementation code before failing tests that the user has approved.

For every task in `tasks.md`, the order is:
1. Write the test from the spec's acceptance criteria.
2. Confirm the test **fails** (red).
3. Get user approval of the test.
4. Write the implementation (green).

The test is generated from the spec, not written alongside the code. If code
and test disagree, the **spec** is the tiebreaker - fix the spec first.

---

## Article IV - CLI First, GUI Second.

> Every capability MUST be reachable headlessly before it is reachable from a
> window.

Rationale: a capability that only exists in a GUI cannot be tested
automatically, cannot be scripted, and locks us into one language. That remains
true, and it is the requirement this article enforces.

**Amended 1.1.0.** The original rationale claimed "none of the incumbent tools can
be used in CI, in a script, or by another program." That is no longer accurate.
GitKraken ships `gk`, a public-preview CLI with `--output json`, git passthrough,
and an MCP server, on macOS, Windows, and Unix. Headless availability is
therefore **parity, not our differentiator**, and no spec may claim it as an
advantage.

What survives as a real, testable edge is narrower:

- **No account, no telemetry, no platform dependency.** `gk` is tied to
  GitKraken's platform, authentication, and issue providers. Ours is not.
- **One surface, not two.** `gk`'s commands are not the surface its GUI uses. Ours
  is the same surface headless and graphical, so a scripted path and a clicked
  path cannot diverge.
- **We use the Git the user already has.** GitKraken bundles and upgrades its own
  build; Fork ships an internal instance. Ours shells out to the binary on the
  machine, which makes equivalence verifiable by a differential test.

A spec that claims a headless advantage over an incumbent must name which of
these three it means, and back it with a test.

Consequences:
- The core exposes a command surface; the GUI is a client of it.
- The GUI MUST NOT invoke `git` directly. It calls the core. If the GUI needs a
  capability the core does not expose, the core grows - the GUI never forks the
  logic.
- Headless is not a separate build. It is the same process without a window.
- This is why the core begins as an in-process module rather than a daemon:
  the *interface* is separated first, the *process boundary* is not forced
  prematurely.

Enforcement: no `git` spawn in any file under `apps/`, `crates/ui/`.

---

## Article V - Unusual Repository States Are Normal.

> Every one of the following MUST be detected, displayed honestly, and
> recoverable. None may crash, hang, or silently corrupt.

The list is a hard requirement, not a backlog:

| State | Detected via |
| --- | --- |
| Rebase in progress | `.git/rebase-merge/`, `.git/rebase-apply/` |
| Merge conflict | `MERGE_HEAD` present |
| Cherry-pick in progress | `CHERRY_PICK_HEAD` |
| Revert in progress | `REVERT_HEAD` |
| Bisect in progress | `BISECT_LOG` |
| Detached HEAD | `HEAD` is not a symref |
| Partial clone | `remote.*.promisor` / `partialclonefilter` |
| Sparse checkout | `core.sparseCheckout` |
| Reftable backend | `extensions.refStorage` |
| Submodule conflicts | `.gitmodules` + `git submodule status` |
| Corrupt or missing reflog | absent `logs/` directory |

Consequences:
- Repo state is probed on open and after every operation.
- When in an in-flight state, the UI MUST show a recovery affordance naming
  the exact git command that resolves it.
- The tool MUST NOT hide or normalize a broken state to keep its own logic
  simple. Showing an honest error beats showing a wrong graph.

Rationale: this is where the incumbents break most visibly, and it is
reproducible, so it is testable.

---

## Article VI - Performance Is A Contract With Numbers.

> The following budgets are part of the specification, not aspirations. Each
> has an automated benchmark that fails the build on regression.

Measured baselines from the validated spike (llvm-project, 599,555 commits,
partial clone, Intel HD 4600):

| Budget | Threshold | Measured |
| --- | --- | --- |
| Extract 600k commits from git | < 30 s | 9.7 s |
| In-memory graph size | < 32 MB | 9.6 MB |
| Load graph into renderer | < 2 s | 522 ms |
| Frame time p50 while traversing | < 16.7 ms (60 fps) | 16.7 ms |
| Frame time p99 while traversing | < 33.4 ms (30 fps) | 19.5 ms |
| Worst single frame | < 50 ms | 20.2 ms |

Consequences:
- Performance is measured by the same headless harness that validated the
  spike, run in CI against a fixture repository.
- The graph is stored in typed arrays in CSR layout. A regression to per-commit
  JS objects fails the budget.
- Any new feature that touches the render path must re-run the benchmark.
- The 600k-commit fixture is a hard requirement, not a nice-to-have. A tool
  that only works on small repos does not beat the incumbents.

---

## Article VII - Simplicity. Three Crates, Three Targets, No More.

> The initial structure is at most three crates. Anything more requires
> documented justification in the plan.

- `crates/core` - Git operations, repository state, graph model. No UI.
- `crates/ui` - renderer and interaction. No `git` spawns (Article IV).
- `apps/desktop` - the window. Shells the binary, wires input to the core.

Adding a crate is an architectural decision requiring a plan amendment. Wrapping
a library to change its shape is forbidden - use it directly (Article VIII).

---

## Article VIII - Anti-Abstraction. Use The Tool Directly.

> A wrapper that only re-exports or reshapes an existing API is forbidden.

Rationale: wrappers accumulate, drift from the upstream API, and hide failures.
When a library's ergonomics are genuinely bad, fix it at our call site or
replace the library - do not add a layer.

Consequences:
- No `GitClient` trait with one implementation.
- No repository-pattern over the graph model.
- No DI container, no service locator, no event-bus framework.
- Error handling is one documented enum at the core boundary, converted once
  at the edge.

---

## Article IX - Integration First. Real Repositories, Not Mocks.

> Git behavior MUST be verified against real repositories in real states.
> Mocking Git is forbidden except to force a failure git cannot naturally
> produce.

Rationale: the incumbents' bugs live in real repository states - conflicted
merges, in-flight rebases, partial clones. A mocked Git cannot reproduce them,
so a mock-heavy test suite passes while the product is broken.

Consequences:
- The test suite builds genuine fixture repositories in a temp directory.
- Every state in the Article V table has a fixture and a test.
- Integration tests run against the real `git` binary, version recorded in the
  test output.
- CI runs the suite across at least Windows and Linux before release.

---

## Governance

### 4.1 Compliance

Every `plan.md` includes a **Constitution Check** section. The `sdd-review` agent
fails any plan or implementation that violates an article without a written,
specific justification.

### 4.2 Amendment

Amendments require:
1. Written rationale for the change.
2. An assessment of backwards compatibility with existing specs and code.
3. User approval.
4. A version bump and a dated entry in SS2.

Principles are immutable by default. Their *application* may evolve.

### 4.3 Priority

When principles conflict, they resolve in article order: earlier articles
outrank later ones. Article I outranks Article VII - a simple design that shells
out to git beats an elegant design that reimplements it.

---

## Amendment log

| Version | Date | Change | Rationale |
| --- | --- | --- | --- |
| 1.0.0 | 2026-09-29 | Initial ratification | Project inception |
| 1.1.0 | 2026-09-29 | Article IV rationale narrowed; requirement unchanged | GitKraken ships `gk`, a public-preview CLI with `--output json` and git passthrough. The claim "none of the incumbent tools can be used in CI, in a script, or by another program" was false. Headless availability is parity. The requirement still binds; only its justification was overreaching, and leaving it would have let every future spec inherit a false premise. Backwards compatibility: no spec, plan, or code referenced the false sentence. FR-013, NFR-004, and NFR-005 in `specs/001-headless-git-operations` depend on the *requirement*, which is unchanged. |
