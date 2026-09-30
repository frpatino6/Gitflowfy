# Implementation Plan: Headless Git Operations

**Branch**: `001-headless-git-operations`
**Spec**: [spec.md](spec.md)
**Status**: Draft
**Created**: 2026-09-29
**Executable**: `gitflowfy` (chosen here; the spec deliberately left it open)

---

## Phase -1: Pre-Implementation Gates

> These gates exist to stop plausible-looking over-engineering. A failed gate
> is either resolved or written into Complexity Tracking with a reason.

### Constitution Check

| Article | Binding here? | How this plan satisfies it |
| --- | --- | --- |
| I - Shell out | **Yes, totally** | This feature *is* Article I. The only way a repository changes is `Command::new("git")` in `crates/core`. No `libgit2`, no `gix`, no object parsing. Every command is enumerated literally in [implementation-details/git-operations.md](implementation-details/git-operations.md). CI gate G4 greps for git spawns outside `crates/core` and G5 fails on an embedded-Git crate in `cargo tree`. |
| II - Reflog only | **Yes, under pressure** | FR-004 requires an audit log. The risk is that it grows into a state tracker. The separation is structural, not a promise: `AuditRecord` holds cwd, argv, exit code, duration and *nothing else* -- no OIDs, no ref names, no file states. It is not readable as repository state even accidentally, and `AuditLog` exposes no query API that could answer "what is the repo like now". Phase 1e adds a test that the record type cannot be extended with repository state without failing review. |
| III - Test first | **Yes, totally** | No product code exists until Phase 0's harness passes. The gates and the differential harness are built from a test-only git runner, so the thing that will test the operation surface is not built by the operation surface. |
| IV - CLI first | **Yes, totally** | Feature 001 ships zero UI. `gitflowfy` reaches every capability. The core is a library *and* a binary target in `crates/core`, so "headless" and "the library" are the same code, not two builds. |
| V - Odd repo states | **Partly** | Six baseline states are in scope (linear, merged, octopus, orphan, detached, empty) and get fixtures. The remaining Article V states are deferred by the spec; for those the binding obligation is narrow: return Git's exit code and output, record it, do not hang. Phase 1f tests that obligation for every deferred state, not just the in-scope ones. |
| VI - Perf budgets | **Yes, partly** | The graph budgets are untouched -- this feature does not touch the render path. The binding number here is SC-010: 5 ms median overhead. Gate G2 measures the *chosen stack's wrapper overhead*, not raw git, so the budget is validated against the thing that must actually meet it. |
| VII - <=3 crates | **Yes** | Feature 001 ships **one**: `crates/core`. `crates/ui` and `apps/desktop` are reserved by the constitution but not created by this feature. Count = 1 of 3. See Project Structure. |
| VIII - Anti-abstraction | **Yes** | No `GitClient` trait, no single-impl traits, no DI. `GitCommand` is a plain struct passed by value. The only error type is `GitflowError`, converted once at the CLI edge. No wrapper reshapes Git's API -- the surface *is* `argv`. |
| IX - Integration first | **Yes, totally** | Every fixture is a real repository built by real `git`. No mocking on either side of a differential comparison. CI runs Windows and Linux, and the run records the git version (FR-012). |

- [x] Every applicable article is satisfied or justified
- [x] No `libgit2`/`gix`/`jgit` in the dependency tree (gate G5)
- [x] No `git` spawn outside `crates/core` (gate G4)

> **Note on gates and phases**: the gates below belong to Phase 0 of the
> implementation phase list in this document. They run before any product code
> exists, using a test-only git runner rather than the operation surface.

### Gate G1: git is usable and identified

**Pass criterion**: `git version` parses, its value is recorded in the build's test
output, and a scratch repository can be created in a temp directory.

**Why first**: the entire feature is downstream of a working `git`. If this fails,
every later gate is meaningless.

### Gate G2: the 5 ms budget is physically achievable

**Pass criterion**: measure **our wrapper's** per-invocation overhead, not git's
runtime. Concretely: 200 sequential invocations of `git rev-parse --git-dir`
dispatched through `GitCommand`, minus the same 200 invoked directly, on this
machine. Report median and p99 of the difference. The budget is satisfiable only
if the difference is at or under 5 ms median.

**Correction made during review.** This gate previously measured 200 raw
`git rev-parse` spawns. That is the wrong measurement and it was the wrong basis
for the stack decision in `research.md`: the baseline is common to every candidate,
so measuring it cannot rank them. A gate that measures the reference point cannot
decide which candidate is nearer to it. The gate now measures the candidate.

**Why this is a gate and not a phase**: SC-010 is a number the spec committed to. If
measurement shows the budget is unreachable on Windows with the chosen stack, that is
a finding to escalate *before* writing the operation surface, not a threshold to
quietly relax later. NFR-002 permits a revision with the measurement attached, but
the revision must be deliberate.

**Scope limit, stated honestly**: G2 runs in Rust because the core is Rust. It
validates that the committed stack meets the committed budget. It does **not** prove
Node would have failed, and no such claim is made. If the Rust margin turns out to
be large, the single-language option in `research.md` remains live and is revisited
at feature 002, not defended here.

### Gate G3: fixture OIDs are reproducible

**Pass criterion**: build the linear fixture twice in two different directories and
assert byte-identical commit OIDs. Repeat for all six baseline shapes.

**Why this is the most important gate**: the entire differential harness compares
against a Git-produced repository. If OIDs are not reproducible, SC-009 fails and
every comparison in Story 3 is noise. Three specific known hazards:

| Hazard | Mitigation |
| --- | --- |
| Windows `core.autocrlf=true` rewrites blob content | Pin `core.autocrlf=false` and `core.eol=lf` per repository |
| Author/committer identity and timestamps vary per machine | Pin `GIT_AUTHOR_*`, `GIT_COMMITTER_*`, and dates to fixed values |
| GPG signing prompts or fails without a key | `commit.gpgsign=false` |
| Git localized output differs by `LANG` | `LC_ALL=C`, `LANG=C` -- also required for stderr comparison, see Risk R4 |

**This gate is expected to fail on first run** and is fixed by the pinning policy
FR-011 already specifies. It must be observed failing before it is fixed.

### Gate G4: no git spawn outside the core

**Pass criterion**: `scripts/check-no-git-outside-core.sh` (and `.ps1` for Windows)
exits 0. It greps every source file outside `crates/core` for `Command::new("git")`,
`process.spawn(`, and `exec("git`, and fails the build on any hit. It is a CI step,
not a review convention.

### Gate G5: no embedded Git implementation

**Pass criterion**: `cargo tree --all-features` output contains no `libgit2`, `gix`,
`gix-*`, `jgit`, or `git2`. The check is a grep over `cargo tree` output in CI.

### Simplicity Gate (Article VII)

- [x] At most 3 crates total -- this feature creates 1
- [x] No new crate added
- [x] No "we may need this later" constructs

### Anti-Abstraction Gate (Article VIII)

- [x] No single-implementation traits
- [x] No DI container or service locator
- [x] No wrapper that merely reshapes an existing API

The one external crate that is worth naming here: `clap` for argument parsing. It
parses argv; it does not wrap Git. Recorded in Complexity Tracking.

### Integration-First Gate (Article IX)

- [x] Fixtures are real repositories built in a temp dir
- [x] Every Article V state in scope has a fixture (six in scope, six deferred with
      a narrower obligation that Phase 1f still tests)
- [x] No Git mocking except to force unnatural failures

### Gate Result

**PASS WITH EXCEPTIONS** -- see Complexity Tracking. The exceptions are all
dependencies, not architectural additions.

---

## Implementation Phases

> Article III: within every phase, the test is written and observed failing before
> the implementation. The phases below are the authoritative ordering; this table
> replaces phase references that earlier drafts of this document left dangling.
> `tasks.md` is the executable decomposition and carries the task numbers.

| Phase | Content | Exit gate |
| --- | --- | --- |
| 0 | Safety net. A test-only git runner, the pinning policy, six fixture builders, five observables, the differential harness, the deferred-Article-V obligation test, and the three CI enforcement scripts. **No product code.** | **G1-G5 all pass.** G3 proves fixture OIDs are reproducible; the harness proves it detects a deliberate divergence |
| 1a | The spawn primitive. `GitCommand` -> `Invocation`, the one and only spawn site | Core spawn tests green |
| 1b | Refusal and honest failure. `remote.rs`, `version.rs`, `error.rs` | Refusal, verbatim-stderr, and missing-git tests green |
| 1c | Byte fidelity and cancellation | Round-trip and no-repair tests green |
| 1d | Concurrency. `lock.rs` | Serialization, parallelism, and no-deadlock tests green |
| 1e | The audit record. `audit.rs` | Record-shape tests green, including the compile-fail guard on repository state |
| 1f | The differential matrix: six fixtures x read/mutate/fail | **0 differences on every observable, every fixture** |
| 2 | The headless surface. `cli/`, `--json`, exit codes, one error conversion | Every capability reachable with no display; CLI suite green |
| 3 | Graph and render | **Not applicable to this feature.** Begins at feature 002, where Article VI is re-measured |
| 4 | UI | **Not applicable to this feature.** Feature 001 ships zero GUI by design |
| 5 | Integration: CI on Windows and Linux, the SC-010 measurement, constitutional audit | Full suite green on both platforms; audit clean |

**Why Phase 0 contains no product code.** The differential harness runs operations
*through our surface*, so building it before the surface exists is circular. Phase 0
therefore uses a test-only git runner -- a helper that calls `git` directly from test
code -- to build fixtures and to read observables. The surface lands in Phase 1a and
is tested by a harness that does not depend on it. An earlier draft had the surface
before the fixtures, which would have let the harness depend on the code under test.

---

## Technical Approach

### Stack decision

**Chosen: Rust for `crates/core`; TypeScript in a web renderer for `crates/ui`,
hosted by Tauri v2 in `apps/desktop`.**

The reasoning, and the part that was genuinely contested:

The spike in `spike/` is JavaScript, and it is validated: 599,555 commits, 9.7 s
extract, 9.6 MB graph, 522 ms renderer load, p99 19.5 ms. Throwing that away costs
real time. So the first instinct is to keep everything in Node.

Two things push against it, and one pulls for it:

1. **SC-010 is the binding constraint, and process spawn is the dominant term.** The
   budget is 5 ms *median overhead over raw git*, and our overhead is dominated by
   how the process is dispatched. The expectation is that Rust's
   `std::process::Command` has lower per-call overhead than Node's
   `child_process.spawn` on Windows, and the differential harness spawns hundreds of
   git processes per run, which is the pattern that amplifies any per-call
   difference. **This expectation is not yet measured.** Gate G2 measures the
   chosen stack directly; until it runs, the Rust choice is a reasoned default with
   a stated bias toward margin, not a measured conclusion.

2. **The graph builder has to live where the graph is built, and the render-path
   headroom is not unlimited.** `spike/build-graph.mjs` streams `git log`, parses,
   and fills typed arrays. The port is mechanical: the algorithm is proven, the CSR
   layout and the five typed arrays (`n`, `offset`, `prow`, `plane`, `lane`) carry
   over unchanged, and the binary format carries over byte-for-byte so the
   already-validated renderer keeps working against it. Whether the ported builder is
   faster or slower than the 9.7 s baseline is **not claimed here**; it is
   re-measured at feature 002 under Article VI before any number is claimed.

3. **What pulls the other way, honestly: the spike's renderer stays JavaScript.**
   This is not discarded work. Canvas with typed arrays is what was measured at
   p99 19.5 ms, and Tauri hosts a webview, so the renderer design survives intact.
   The port cost falls on `build-graph.mjs` only, and that script belongs to the
   graph feature (002), not to this one.

**Electron was rejected** on footprint and on the spawn budget: it ships a Chromium
per instance, competing for the same memory the 9.6 MB graph and the render loop are
trying to protect, and it buys nothing here that Tauri does not.

**The split is real and intentional**: Rust core, TypeScript renderer. The interop
boundary is Tauri's command surface, and Article IV already requires the interface to
be separated before the process boundary is forced, so this is the boundary the
constitution asked for. The graph crosses it as raw bytes, not JSON, which is the
whole reason the CSR layout is a binary blob rather than an object graph.

**For feature 001 specifically, none of this is load-bearing yet.** This feature is
the core only, and it is headless. The renderer decision binds from feature 002
onward. What binds now is the spawn budget, and that is why the core is Rust.

### Architecture

```
crates/core/
  git/          the ONLY place that spawns git
    command.rs    GitCommand { repo, args } -> Invocation { argv, exit, stdout, stderr, duration }
    audit.rs      AuditRecord, AuditLog  (append-only log in the git dir; never repository state)
    lock.rs       per-repository serialization (FR-020)
  fixtures/     real repository builders, six shapes
  differential/  the harness: run both sides, compare observables
  cli/          argv parsing, output shaping, the one error conversion
```

Data flow for a single operation: caller builds a `GitCommand` -> the per-repo lock
serializes -> the git directory is resolved by `git rev-parse --git-dir` -> a refused
verb stops here without a spawn -> `Command::new("git")` spawns with stdin closed and
a pinned environment -> stdout/stderr captured as raw bytes -> duration measured ->
one `AuditRecord` appended to the tool's log in that git directory -> the
`Invocation` returns to the caller. Nothing in that path interprets Git's output.

### Git Operations Required

Full literal enumeration with flags, exit-code behavior, and the observables each
operation is compared on:
[implementation-details/git-operations.md](implementation-details/git-operations.md).

The mutating surface, at a glance:

| Operation | Command | Notes |
| --- | --- | --- |
| Stage | `git add -- <pathspec...>` | `--` prevents path-as-flag injection |
| Commit | `git commit -m <msg> -- <pathspec...>` | |
| Branch create | `git branch <name>` | |
| Branch delete | `git branch -D <name>` | |
| Switch | `git switch <branch>` | fallback `git checkout <branch>` if git < 2.23 |
| Detach | `git checkout --detach <ref>` | needed for the detached-HEAD fixture |
| Merge | `git merge --no-ff <ref>` | `--no-ff` so the merged fixture is reproducible |
| Octopus merge | `git merge --no-ff <a> <b> <c>` | |
| Tag | `git tag <name> [<ref>]` | |
| Reset | `git reset --soft\|--mixed\|--hard <ref>` | |
| Stash | `git stash push -m <msg>` / `git stash pop` | |
| Revert | `git revert --no-edit <oid>` | |
| Cherry-pick | `git cherry-pick <oid>` | |
| Remove | `git rm -- <path>` | |
| Rename | `git mv <src> <dst>` | |

The comparison observables, which are how the harness proves equivalence:

| Observable | Command |
| --- | --- |
| Refs and OIDs | `git for-each-ref --format="%(refname) %(objectname)"` |
| HEAD resolution | `git rev-parse HEAD` plus `git symbolic-ref -q HEAD` -- **three** distinct states, see below |
| Index, including conflict stages | `git ls-files -s` |
| Working tree | `git status --porcelain=v2 --untracked-files=all` plus content hashes of every file the index does not track |
| Reflog | `git reflog --format="%H %gs"` -- deliberately not raw, see R4 |

### Headless Surface

```bash
gitflowfy git <repo-path> -- <argv...>       # run any git command, return its result
gitflowfy audit <repo-path>                  # read the invocations this tool performed
gitflowfy audit <repo-path> --clear          # apply the documented retention policy now
gitflowfy diff <baseline-repo> <candidate>   # the differential comparison
gitflowfy fixtures build <shape> <path>      # build a real fixture repository
gitflowfy version                            # tool version plus the git version found
```

`gitflowfy audit` reads the persisted log, so it works from a fresh process. That is
the whole reason FR-004 requires persistence; with the in-memory design this
subcommand would have been decorative.

**The persistence decision has a cost, and it is not free.** One file append per
invocation is now inside the 5 ms SC-010 budget, on Windows, on the measured path.
Opening, writing, and closing a file per invocation is not free, and if G2 shows the
budget is missed *because of* the write, the options are a retained file handle, a
buffered writer flushed on exit, or an explicit NFR-002 revision with the number
attached. None of those is a quiet adjustment to the budget. This is why
`t_overhead_includes_the_audit_write` exists: a design that meets 5 ms with the
write stubbed out has not met 5 ms.

Every subcommand supports `--json`. Text is for a human at a terminal; JSON is the
machine surface a later GUI or a CI job consumes. The core returns raw bytes in both
cases -- the shaping happens once, at the edge (Article VIII).

| Command | Output | Used by |
| --- | --- | --- |
| `git` | exit code, stdout, stderr, duration | every later feature; the GUI |
| `audit` | ordered invocations, one per command run | the user; a bug report |
| `diff` | per-observable pass/fail, naming what differed | CI, the developer |
| `fixtures` | a real repository on disk | every later test |
| `version` | tool and git versions | FR-012, CI logs |

`gitflowfy git` deliberately does **not** restrict the command set (FR-025). The
supported set is whatever the installed Git supports. Remote and network operations
are the only refusal, per FR-015.

---

## Requirement Traceability

> Every requirement from the spec maps to a technical decision and a test.
> A requirement with no test is not implemented.

| Requirement | Technical decision | Verified by |
| --- | --- | --- |
| FR-001 | Every mutation is `Command::new("git")` in `crates/core/git/command.rs` | `differential::no_embedded_git` + CI gate G5 (`cargo tree` grep) |
| FR-002 | `GitCommand { repo: PathBuf, args: Vec<OsString> }`; no free-form string is ever shell-parsed | `t_argv_is_not_parsed`: an arg containing `;` and `&&` reaches git as one literal arg |
| FR-003 | `Invocation { exit_code, stdout: Vec<u8>, stderr: Vec<u8>, duration: Duration }` | `t_result_carries_all_four` |
| FR-004 | One `AuditRecord { cwd, argv, exit_code, duration }` per invocation, appended to a log the tool owns inside the git dir, resolved by asking `git rev-parse --git-dir` rather than assuming a `.git` path | `t_one_record_per_invocation`; `t_git_dir_is_asked_not_assumed`; `t_record_on_failure` |
| FR-005 | `AuditLog` appends in issue order under the per-repo lock; one JSON object per line; a record from a concurrent process does not interleave | `t_records_are_in_issue_order`; `t_concurrent_appends_do_not_corrupt_a_line` |
| FR-006 | Non-zero exit returns `Err(GitflowError::GitFailed { .. })` carrying git's untouched bytes; no `map_err` rewrites them | `t_git_error_text_is_verbatim` |
| FR-007 | `differential::compare(left, right, op)` runs the op twice from identical fixtures | `t_differential_*` across all six fixtures |
| FR-008 | Compares refs, HEAD+attached/detached, index, working tree, reflog; state exact, output content-compared with path normalized | `t_compares_all_five_observables`; `t_stdout_normalizes_only_the_path` |
| FR-009 | A mismatch names the observable and both values | `t_mismatch_names_the_observable` |
| FR-010 | Six builders in `crates/core/fixtures/` | `t_each_fixture_has_its_shape` (6 tests) |
| FR-011 | `fixtures::pin()` sets identity, both dates, TZ, `LC_ALL=C`, `autocrlf=false`, `eol=lf`, `gpgsign=false` | Gate G3; `t_oids_are_reproducible` |
| FR-012 | `gitflowfy version` and every test-run preamble print `git version` + platform | `t_run_records_git_version` |
| FR-013 | The `gitflowfy` binary reaches every capability; the core is a lib *and* a bin target | `t_cli_reaches_every_capability` |
| FR-014 | Exit 0 on success, 1 on our own refusal/error, and git's code always present in the output | `t_exit_code_distinguishes`; `t_git_code_present_in_output` |
| FR-015 | A remote/network verb list is refused before spawn, with no process started | `t_fetch_is_refused_without_spawning` (asserts no audit success record) |
| FR-016 | `AuditRecord` has no field capable of holding repository state; `AuditLog` has no state-query method | `t_audit_type_exposes_no_repository_state` (compile-time: constructing a record from repo state does not compile) |
| FR-017 | Every rendered audit output is labeled "invocations performed by this tool", and never "history" | `t_audit_output_is_labelled` |
| FR-018 | Bytes stored raw; `--text` produces a derived string, and a lossy decode sets `lossy: true` in JSON | `t_bytes_survive_roundtrip`; `t_invalid_utf8_sets_lossy_flag` |
| FR-019 | Cancellation kills the child; then re-probes with `rev-parse`/`status` and returns the observed state marked cancelled. No repair. | `t_cancel_leaves_git_state_intact`; `t_cancel_does_not_repair` |
| FR-020 | A per-repo `Mutex<()>` keyed by canonicalized path; cross-repo operations run in parallel threads | `t_same_repo_serializes`; `t_different_repos_parallel` |
| FR-021 | Non-zero exit is always a non-success `Invocation`; no per-command semantics anywhere in the crate | `t_nonzero_is_always_non_success`; a `differential` run of `diff --quiet` proves the code passes through |
| FR-022 | Startup probes `git version`; missing/unparseable is a plain error, and the found version is recorded and compared to the validated one | `t_missing_git_fails_plainly`; `t_version_mismatch_is_reported` |
| FR-023 | No mocking in the differential harness; fixtures are real | `t_harness_uses_real_repositories` |
| FR-024 | No object model. Every fact comes from a `git` call | Covered by G5 plus a review check that no code parses `.git/objects` |
| FR-025 | No allowlist. `gitflowfy git` forwards argv, with stdin closed | `t_unlisted_command_runs` (e.g. `notes --ref=todo list`) |
| FR-026 | Empty repository is the sixth fixture; `rev-parse` on an unborn HEAD returns git's own code | `t_empty_repo_fixture`; `t_unborn_head_returns_git_code` |
| NFR-001 | No render path touched; graph budgets untouched and explicitly not claimed | `scripts/check-perf-scope.sh` asserts the graph modules are unchanged this feature; constitution Art. VI re-measured at feature 002 |
| NFR-002 | Overhead measured per invocation by the harness, median and p99, per platform | `t_overhead_within_budget` (fails over 5 ms median); gate G2 |
| NFR-003 | CI matrix: windows-latest and ubuntu-latest, both required to pass | CI job definition; `t_run_records_platform` |
| NFR-004 | No window is created; the binary never links a UI toolkit | `t_runs_with_no_display` sets `DISPLAY=""` and runs the full CLI suite |
| NFR-005 | Feature 001 delivers only the headless half; no UI crate is created | Project Structure shows 1 of 3 crates |
| NFR-006 | The invariant the whole harness exists to test | `differential::*` -- the invariant is the test suite, not a separate assertion |
| FR-027 | A documented size cap with a documented retention policy; only the tool's own log file is ever truncated; the truncation is reported, never silent | `t_log_is_bounded`; `t_truncation_touches_only_our_file`; `t_truncation_is_reported` |
| FR-028 | The audit write is inside the measured path: overhead is measured with the append in place, not with it stubbed out | `t_overhead_includes_the_audit_write` |

---

## Performance Impact

| Metric | Budget | Baseline | New expected | Measured |
| --- | --- | --- | --- | --- |
| Per-invocation overhead, median | 5 ms (SC-010) | 0 (raw `git` is the reference) | <= 5 ms | gate G2 + `t_overhead_within_budget` |
| Per-invocation overhead, p99 | 25 ms (SC-010) | 0 | <= 25 ms | same |
| Audit append cost | **not a separate budget -- it is inside the two rows above** | 0 | n/a | `t_overhead_includes_the_audit_write` |
| Extract 600k commits | < 30 s | 9.7 s | not touched | not re-measured |
| Graph size | < 32 MB | 9.6 MB | not touched | not re-measured |
| Renderer load | < 2 s | 522 ms | not touched | not re-measured |
| Frame p50 / p99 / max | 16.7 / 33.4 / 50 ms | 16.7 / 19.5 / 20.2 ms | not touched | not re-measured |

Benchmark command:
```bash
cargo test --package gitflowfy-core -- --nocapture overhead
```

The last five rows are listed so their absence is visible. This feature touches none
of them, and per NFR-001 it claims no number for any of them. They are re-measured
with `node --experimental-websocket spike/run-spike.mjs` when the graph feature lands.

---

## Project Structure

```
crates/
  core/
    Cargo.toml
    src/
      lib.rs
      git/
        mod.rs
        command.rs     GitCommand -> Invocation; the only spawn site
        audit.rs       AuditRecord, AuditLog; the tool's own log in the git dir
        lock.rs        per-repository serialization
        remote.rs      the FR-015 refusal list
        version.rs     git version probe
      error.rs         GitflowError, the single error type
      fixtures/
        mod.rs         pin(), and the six builders
      differential/
        mod.rs         the harness
        observables.rs the five comparisons
        overhead.rs    the SC-010 measurement
      cli/
        mod.rs         argv parsing and output shaping
        bin/gitflowfy.rs
  ui/                  (reserved by Art. VII, NOT created by this feature)
apps/
  desktop/             (reserved by Art. VII, NOT created by this feature)
```

Note on the constitution's wording: Article VII says `apps/desktop` "shells the
binary", while Article IV says the core begins as an in-process module rather than a
daemon. Read together, `apps/desktop` is the window crate, and the core is compiled
*into* the desktop binary rather than running beside it as a separate process. That
satisfies both. Flagged here because the wording admits a second reading.

| Path | Responsibility |
| --- | --- |
| `src/git/command.rs` | The one and only place a `git` process starts (G4) |
| `src/git/audit.rs` | Append-only invocation log; no repository state (FR-016) |
| `src/git/lock.rs` | One operation at a time per repository (FR-020) |
| `src/git/remote.rs` | Refuses network verbs before any spawn (FR-015) |
| `src/error.rs` | `GitflowError`; converted once, at the CLI edge |
| `src/fixtures/mod.rs` | Deterministic real repositories, six shapes (G3) |
| `src/differential/observables.rs` | The five observable comparisons (FR-008) |
| `src/differential/overhead.rs` | Measures the SC-010 budget |
| `src/cli/` | argv, `--json`, exit codes (FR-013, FR-014) |
| `scripts/check-no-git-outside-core.ps1` | Gate G4 |
| `scripts/check-no-embedded-git.sh` | Gate G5 |

---

## Complexity Tracking

| Exception | Article | Justification | Revisit when |
| --- | --- | --- | --- |
| `clap` dependency for argv parsing | VIII | Parses argv; it does not wrap or reshape Git's API. Hand-rolling nested subcommands, `--json` flags, and help text is ~400 lines of code whose only value would be avoiding a dependency, and it would be worse code. | If clap's MSRV or license ever conflicts with a release requirement. |
| `serde` + `serde_json` for `--json` output | VIII | One serialization boundary, at the edge, exactly where Article VIII says conversion happens. Not a wrapper over Git. | If the JSON shape is replaced by something non-JSON (it will not be). |
| `tempfile` for fixture directories | IX | Article IX requires real repositories in a temp dir. Hand-rolling temp dir creation and guaranteed cleanup on Windows is a bug farm, and a leaked fixture directory is a real hazard. | Never; this is the correct use. |
| Deferred Article V states | V | The spec scopes this feature to six baseline shapes. For the other states the narrow obligation (return git's code, record it, do not hang) is still tested in Phase 1f. The recovery affordance the article ultimately requires needs a UI or an interactive CLI, neither of which exists yet. | The repository-state feature, which must deliver all eleven. |
| Audit log persisted, not in memory | II | **Corrected during review.** The original plan kept the log in memory, which is incompatible with Story 2: an in-memory log dies with its process, so "auditable after the fact" from a fresh `gitflowfy audit` would have returned an empty list every time. Persistence does not breach Article II, because the record holds four invocation fields and no repository state - what the article forbids is a model of the repository, not a log of what this tool ran. The log is a file the tool owns, inside the git dir Git itself reports, bounded by FR-027. | When the user asks for cross-machine or historical audit, at which point the retention policy needs a real size rather than a placeholder. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| R1: Fixture OIDs differ between Windows and Linux, so the differential matrix cannot compare across platforms | **High** | Blocks the harness entirely | Gate G3 pins every input. If it still fails, the differential comparison runs *within* a platform (ours vs. raw git on the same machine), which is the actual requirement; cross-platform OID equality is a bonus, not the invariant. |
| R2: Windows path handling breaks on non-ASCII paths, spaces, or `\\?\` prefixes | Medium | Fixture or command failures in the wild | The spec requires opaque path handling (Edge Cases). Test with a fixture path containing a space and a non-ASCII character. Pass paths to `Command` as `Path`, never as strings. |
| R9: A HEAD-state comparison written against the exit code of `symbolic-ref` alone treats an unborn HEAD as a normal one | Medium | The empty-repository fixture would compare equal to a populated repository, silently | Verified against a real `git`: an unborn HEAD makes `symbolic-ref` **exit 0** while naming a branch that does not exist. The observable records the `rev-parse` exit code, its output, and the `symbolic-ref` exit code and output together. See `git-operations.md` 5.2. |
| R3: Per-repo mutex deadlocks if an operation re-enters the lock | Low | Hangs a test run, which is the one thing the spec forbids | The lock is held by a leaf function that only spawns and collects. No re-entrant path exists; add a test that asserts two serialized operations complete in order. |
| R4: Git's stderr is localized, so the two sides of a comparison differ | **High** on a localized Windows, low on CI | False differential failures that look like real bugs | `LC_ALL=C`, `LANG=C` in `fixtures::pin()` and in the differential harness environment. Also, reflog comparison uses `%H %gs` and never the raw line, which contains timestamps that legitimately differ between two runs. |
| R5: A Git release changes output format and the differential matrix goes red on a machine nobody controls | Medium | CI noise, loss of trust in the harness | FR-012 records the version. A mismatch against the validated version is reported distinctly from a behavioral failure, so a version change is diagnosable rather than mysterious. |
| R6: SC-010 turns out unreachable on some Windows configuration | Medium | Requires an NFR-002 revision | Gate G2 measures before the promise is built on. The revision path requires the measurement attached and explicit approval. |
| R7: The refused-verb list (FR-015) drifts as Git adds network commands | Medium | A new network verb could escape the refusal | The refusal list is a first-party list and therefore necessarily incomplete, and no list can be complete. The real mitigations are a test that asserts a refused verb starts **no process at all** (`t_fetch_is_refused_without_spawning`), and the differential harness, which would fail loudly if a fixture operation left the machine differently. An earlier version of this risk claimed the harness runs "in a temp directory with no network"; that was wrong -- a temp directory is a path, not a network boundary. Recorded in Complexity Tracking as a known weakness of a first-party list. |
| R8: A future feature needs state the CLI cannot express, and the surface grows a private model | Medium | Article II violation by accretion | `AuditRecord` is frozen (FR-016) and the crate has no object model (FR-024). The review gate for any new struct is "can this be replaced by a `git` call?" |

---

## Constitutional Review Outcome

Reviewed against all nine articles. Delegation to `sdd-review` failed in this
session with `Model not found: inherit/.` -- the agent registry is cached at
opencode startup and the `sdd-plan`/`sdd-review` frontmatter has since been
corrected on disk, so the next session will delegate cleanly. The review below was
therefore performed inline, against the same checklist, with the same adversarial
intent. Findings and dispositions:

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | Blocking | `gitflowfy audit` was exposed as a standalone subcommand while the log was in memory, so a fresh process would always read an empty list. Story 2's "after the fact" was unfalsifiable. | **Spec amended.** Q5 rewritten; FR-027 and FR-028 added; `plan.md` CLI, architecture, data flow and Complexity Tracking all updated; `tasks.md` section 1e rebuilt (T050-T060). |
| 2 | Blocking | Six prose references to "Phase 3" and "Phase 4" pointed at sections that did not exist. The plan had no implementation phase list at all. | **Fixed.** An Implementation Phases table now exists with exit gates per phase, and all six references resolve. |
| 3 | Blocking | Gate G2 measured raw `git` spawns -- the baseline common to every candidate -- so it could not rank the candidates the stack decision rested on. | **Fixed.** G2 now measures the chosen stack's wrapper overhead directly. The correction is recorded in the gate text, not silently applied. |
| 4 | Blocking | Four unmeasured performance claims stated as fact ("measurably higher", "sub-millisecond", "becomes faster rather than a regression", "no runtime tax"), violating Article VI. | **Fixed.** All recast as explicitly labelled expectations, and `research.md` now states plainly that no benchmark in this repository supports them. |
| 5 | Medium | Risk R7 claimed the harness runs "in a temp directory with no network". A temp directory is a path, not a network boundary. | **Fixed.** R7 replaced with the real mitigations: a test asserting a refused verb starts no process, plus the differential harness. |
| 6 | Medium | The Article III row claimed the differential harness is built "before any operation surface exists", which is circular. | **Fixed.** Phase 0 now uses a test-only git runner; the harness therefore does not depend on the code it tests. `tasks.md` records this as an improvement over the plan's original order. |
| 7 | Low | FR-014 gives process exit 1 on our own errors while git's code lives only in the output, so a script reading `$?` cannot distinguish "git returned 1" from "we failed". | **Accepted, not changed.** The spec already states the process code is a summary and git's code is preserved verbatim. Noted here so it is a known property, not a surprise. |
| 8 | Low | `fixtures/` and `differential/` live in `crates/core/src/`, so test infrastructure ships in the production library. | **Accepted.** `gitflowfy fixtures build` and `gitflowfy diff` are exposed subcommands, so they are product surface, not dead weight. |
| 9 | Low | The Article V fixture list in the generic `sdd-tasks` agent is wider than this feature's scope. | **Resolved in the spec's favor** and recorded in `tasks.md`. The deferred states keep their narrow obligation, tested at T022. |
| 10 | Low | A partial-clone fixture would need the network, which no fixture may. | **Fixed.** T022 specifies a local-path promisor remote, so the fixture is buildable offline. |

Automated re-check after the fixes: `node .claude/check-sdd-docs.mjs` -- 22 of 22
checks pass, including 34/34 requirement traceability from spec through plan to tasks,
zero test-after-implementation violations, and zero unmeasured performance claims.

**Verdict: the plan may be marked Approved**, subject to the one thing no document
review can settle -- gates G1 through G5 have to actually run. The plan's status
remains Draft until G2 and G3 are observed, because both of those can invalidate
decisions the plan has already made.

---

## Plan Self-Review

- [x] Every FR and NFR is traced to a decision and a test (32 rows: FR-001..026,
      NFR-001..006)
- [x] No `[NEEDS CLARIFICATION]` remains unresolved -- the spec has 0
- [x] Every git command is spelled out literally
      ([implementation-details/git-operations.md](implementation-details/git-operations.md))
- [x] Headless surface defined before GUI -- feature 001 has no GUI
- [x] Fixtures specified for every in-scope Article V state (six, plus a narrower
      obligation tested for the six deferred ones)
- [x] Perf budget and measurement method stated (SC-010 plus gate G2)
- [x] Complexity exceptions are justified, not hand-waved (five, each with a trigger)
- [x] This plan stays high-level; code lives in `implementation-details/`
- [x] Pure ASCII output
