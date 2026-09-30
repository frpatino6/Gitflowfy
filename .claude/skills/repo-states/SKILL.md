---
name: repo-states
description: How to detect, build fixtures for, and handle the unusual Git repository states in Constitution Article V. Use when a feature touches repository state, and whenever a fixture is needed for rebase-in-progress, conflicts, detached HEAD, or similar.
---

# Unusual repository states

Constitution Article V makes these a hard requirement. They are where GitKraken,
Fork, and Extensions break, so they are where our advantage is tested.

## Detection

Probe on open and after every operation. Do not assume a happy path.

| State | Signal |
| --- | --- |
| Rebase in progress | `.git/rebase-merge/` or `.git/rebase-apply/` exists |
| Merge conflict | `MERGE_HEAD` exists |
| Cherry-pick in progress | `CHERRY_PICK_HEAD` exists |
| Revert in progress | `REVERT_HEAD` exists |
| Bisect in progress | `BISECT_LOG` exists |
| Detached HEAD | `HEAD` is a 40-hex oid, not `ref: ...` |
| Partial clone | `remote.origin.promisor` is set |
| Sparse checkout | `core.sparseCheckout` is set |
| Reftable | `extensions.refStorage` is set |
| Corrupt reflog | `.git/logs/` missing or unreadable |

Prefer asking git over probing the filesystem. `git rev-parse --git-dir`,
`git symbolic-ref HEAD`, and `git rev-parse --is-shallow-repository` are more
robust than path checks, and they keep Article I intact.

## Behavior

When an in-flight state is detected:
- Show it plainly. Name the state and the exact command that resolves it.
- Offer recovery: abort, continue, skip - whichever applies to that state.
- Never hide or normalize a broken state to keep internal logic simple.
  An honest error beats a wrong graph.

## Building fixtures

A fixture is a real repository built in a temp directory. Article IX forbids
mocking git to produce these - they are the states a mock cannot reproduce,
which is the whole reason the incumbents' test suites miss them.

### In-progress states

```bash
# rebase in progress: start one that must stop at a conflict
git init repo && cd repo
printf 'a\n' > f && git add f && git commit -m base
git checkout -b feature
printf 'feature\n' > f && git commit -am feature
git checkout main
printf 'main\n' > f && git commit -am main
git rebase feature          # stops on conflict
```

That final command exits non-zero and leaves the repo mid-rebase. That *is* the
fixture. Do not clean it up.

### Merge conflict

```bash
git checkout feature && git merge main   # conflict, MERGE_HEAD present
```

### Detached HEAD

```bash
git checkout --detach HEAD
```

### Partial clone

```bash
git clone --filter=blob:none --no-checkout <url> partial
```

Note: a partial clone fetches blobs on demand, so any code path that reads file
contents must handle a network fetch mid-operation.

### Missing reflog

```bash
rm -rf .git/logs
```

This one tests Article II directly: the tool must say so and offer manual
recovery, never silently fall back to a private tracker.

### Octopus merge

```bash
git checkout -b b2 && git commit --allow-empty -m b2
git checkout -b b3 && git commit --allow-empty -m b3
git checkout main && git merge b2 b3 --no-ff
```

## The differential harness

The highest-value fixture work. For an operation, run it through our core and
through raw `git`, then compare the resulting state. If they differ, we are
wrong.

Compare: ref OIDs, `git status --porcelain` output, `git diff --cached`, and
`git rev-parse HEAD`. Run both in identical starting states.

This is what makes a "it works" claim mean something. Without it, tests only
prove the tests pass.
