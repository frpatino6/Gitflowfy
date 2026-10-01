# T077: Deferred Article V States Carried Forward

**Status:** ✅ CONFIRMED

The following Article V states were explicitly deferred from Feature 001 scope but carry a binding narrow obligation (return Git's exit code, record it, do not hang, do not corrupt). Full recovery affordances are deferred to the **repository-state feature**.

## Deferred States (narrow obligation tested in Feature 001 T022)

| State | Narrow Obligation | Full Affordance Target |
|-------|-------------------|------------------------|
| Rebase in progress | Return git's exit code, record, don't hang | repository-state feature |
| Merge conflict | Return git's exit code, record, don't hang | repository-state feature |
| Cherry-pick in progress | Return git's exit code, record, don't hang | repository-state feature |
| Revert in progress | Return git's exit code, record, don't hang | repository-state feature |
| Bisect in progress | Return git's exit code, record, don't hang | repository-state feature |
| Partial clone | Return git's exit code, record, don't hang | repository-state feature |
| Sparse checkout | Return git's exit code, record, don't hang | repository-state feature |
| Missing reflog | Return git's exit code, record, don't hang | repository-state feature |
| Reftable backend | Return git's exit code, record, don't hang (skipped-with-reason if git < 2.45) | repository-state feature |
| Submodule conflicts | Return git's exit code, record, don't hang | repository-state feature |

## Verification in Feature 001

- **T022**: Tests narrow obligation for all deferred states
- **T077**: This document confirms explicit carry-forward
- **T075**: Platform field recorded in every run (CI cross-platform)

## Next Feature: repository-state

The repository-state feature will implement:
1. Full detection UI for all 11 states
2. Recovery affordances naming exact git commands
3. Integration with differential harness for state-specific tests
4. Full fixture builders for each state

---

**Signed off:** Feature 001 constitutional audit
**Date:** 2026-09-30