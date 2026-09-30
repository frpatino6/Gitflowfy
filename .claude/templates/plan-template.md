---
description: Fill this template when planning implementation. Maps spec requirements to technical decisions.
---

# Implementation Plan: [FEATURE NAME]

**Branch**: `[NNN-feature-name]`
**Spec**: [spec.md](spec.md)
**Status**: Draft | Under Review | Approved
**Created**: YYYY-MM-DD

---

## Phase -1: Pre-Implementation Gates

> These gates exist to stop plausible-looking over-engineering. A failed gate
> is either resolved or written into Complexity Tracking with a reason.

### Constitution Check

| Article | Binding here? | How this plan satisfies it |
| --- | --- | --- |
| I - Shell out | | |
| II - Reflog only | | |
| III - Test first | | |
| IV - CLI first | | |
| V - Odd repo states | | |
| VI - Perf budgets | | |
| VII - <=3 crates | | |
| VIII - Anti-abstraction | | |
| IX - Integration first | | |

- [ ] Every applicable article is satisfied or justified
- [ ] No `libgit2`/`gix`/`jgit` in the dependency tree
- [ ] No `git` spawn outside `crates/core`

### Simplicity Gate (Article VII)

- [ ] At most 3 crates total
- [ ] No new crate added, or justified below
- [ ] No "we may need this later" constructs

### Anti-Abstraction Gate (Article VIII)

- [ ] No single-implementation traits
- [ ] No DI container or service locator
- [ ] No wrapper that merely reshapes an existing API

### Integration-First Gate (Article IX)

- [ ] Fixtures are real repositories built in a temp dir
- [ ] Every Article V state in scope has a fixture
- [ ] No Git mocking except to force unnatural failures

### Gate Result

**PASS** | **PASS WITH EXCEPTIONS** (see Complexity Tracking) | **FAIL**

---

## Technical Approach

### Architecture

<!-- How does this feature sit in the core/ui/apps split? What is the data
     flow? Keep this readable; push detail to implementation-details/. -->

### Git Operations Required

<!-- Constitution Article I. Every mutating operation must be listed as a
     literal command. This is the auditable surface. -->

| Operation | Command | Notes |
| --- | --- | --- |
| | | |

### Headless Surface

<!-- Constitution Article IV. What does this feature expose without a GUI? -->

```bash
# Example shape
gitflow <command> [flags]
```

| Command | Output | Used by |
| --- | --- | --- |
| | | |

---

## Requirement Traceability

> Every requirement from the spec maps to a technical decision and a test.
> A requirement with no test is not implemented.

| Requirement | Technical decision | Verified by |
| --- | --- | --- |
| FR-001 | | |
| NFR-001 | | |

---

## Performance Impact

<!-- Constitution Article VI. State the budget, the measurement method, and
     the current baseline. If the render path is touched, the benchmark must
     be re-run. -->

| Metric | Budget | Baseline | New expected | Measured |
| --- | --- | --- | --- | --- |
| | | | | |

Benchmark command:
```bash
```

---

## Project Structure

```
crates/
  core/
  ui/
apps/
  desktop/
```

<!-- Only the files this feature creates or changes. -->

| Path | Responsibility |
| --- | --- |
| | |

---

## Complexity Tracking

> Every gate exception, every place we knowingly exceed an article.

| Exception | Article | Justification | Revisit when |
| --- | --- | --- | --- |
| | | | |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| | | | |

---

## Plan Self-Review

- [ ] Every FR and NFR is traced to a decision and a test
- [ ] No `[NEEDS CLARIFICATION]` remains unresolved
- [ ] Every git command is spelled out literally
- [ ] Headless surface defined before GUI
- [ ] Fixtures specified for every in-scope Article V state
- [ ] Perf budget and measurement method stated
- [ ] Complexity exceptions are justified, not hand-waved
- [ ] This plan stays high-level; code lives in `implementation-details/`
