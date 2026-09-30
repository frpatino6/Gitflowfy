# Git Operations: the literal surface

Constitution Article I: every mutating operation is a literal `git` invocation, not
a description of one. This file is that enumeration. The differential harness
compares results obtained through this surface against results obtained by running
these exact commands directly.

Every command assumes `-C <repo>` unless stated otherwise. `--` is used wherever a
path could be mistaken for a flag.

---

## 1. Process invocation

```
Command::new("git")
    .arg("-C").arg(repo)
    .args(argv)                 // forwarded verbatim; never shell-parsed
    .stdin(Stdio::null())       // FR-010: a prompting command fails fast, never hangs
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .env("LC_ALL", "C")         // R4: unlocalized output, required for comparison
    .env("LANG", "C")
    .env("GIT_TERMINAL_PROMPT", "0")
    .env_remove("GIT_EDITOR")
    .env_remove("GIT_PAGER")
```

`GIT_PAGER` removal matters: `git log` invoked in a pipeline would otherwise spawn a
pager and the captured stdout would be empty.

There is no `sh -c`, no PowerShell, and no command string anywhere in the crate
(FR-002). A `t_argv_is_not_parsed` test asserts an argument containing `; rm -rf` or
`&&` reaches Git as one literal argument.

---

## 2. Startup probe

```
git version
```

Parsed for the version string. Recorded in every test run (FR-012). A missing,
non-executable, or unparseable Git is a plain failure with no fallback (FR-022).

---

## 3. Mutating operations offered by the surface

| Operation | argv | Notes |
| --- | --- | --- |
| Stage | `add -- <pathspec...>` | |
| Commit | `commit -m <msg> -- <pathspec...>` | |
| Branch create | `branch <name>` | |
| Branch delete | `branch -D <name>` | `-D` so fixture teardown is not interactive |
| Branch list | `branch --list` | read-only |
| Switch | `switch <branch>` | needs git >= 2.23; gate G2 records the version found |
| Switch fallback | `checkout <branch>` | used only if `switch` is unavailable |
| Detach | `checkout --detach <ref>` | |
| Attach | `checkout <branch>` | |
| Merge | `merge --no-ff <ref>` | `--no-ff` so a merge fixture is reproducible |
| Octopus merge | `merge --no-ff <a> <b> <c>` | three or more parents |
| Tag | `tag <name> [<ref>]` | |
| Reset | `reset --soft\|--mixed\|--hard <ref>` | mode is the caller's choice, passed through |
| Stash push | `stash push -m <msg>` | |
| Stash pop | `stash pop` | |
| Revert | `revert --no-edit <oid>` | `--no-edit` keeps it non-interactive |
| Cherry-pick | `cherry-pick <oid>` | |
| Remove | `rm -- <path>` | |
| Rename | `mv <src> <dst>` | |

Per FR-025 this list is **not** an allowlist. It is the documented set this feature
has been differentially validated against. `gitflowfy git` forwards any argv the
installed Git supports; a command not listed here simply has no differential coverage
yet.

### Refused: network operations (FR-015)

```
fetch    pull    push    clone    remote    submodule    ls-remote    archive --remote
```

Refused **before** any process starts. The refusal is a `Refused` error, not a
non-zero exit from Git, and no audit record claims a success. A `t_fetch_is_refused_
without_spawning` test asserts the spawn counter did not move.

---

## 4. Read-only operations

| Purpose | argv |
| --- | --- |
| Is this a repository | `rev-parse --git-dir` |
| HEAD commit | `rev-parse HEAD` |
| Is HEAD symbolic | `symbolic-ref -q HEAD` (exit 1 means detached -- a normal result, not an error) |
| Working tree state | `status --porcelain=v2 --branch --untracked-files=all` |
| History | `log --topo-order --format=...` |
| All refs | `for-each-ref --format="%(refname) %(objectname)"` |
| Index with stages | `ls-files -s` |
| Object contents | `cat-file -p <oid>` |
| Merge base | `merge-base <a> <b>` (absent exit 1 proves an orphan branch) |

---

## 5. The five comparison observables (FR-008)

These are what the differential harness reads. They are the definition of "the same
repository state".

### 5.1 Refs and OIDs -- exact

```
git for-each-ref --format="%(refname) %(objectname)"
```

Sorted output, compared byte for byte. No normalization: an OID is an OID.

### 5.2 HEAD state -- exact

```
git rev-parse HEAD
git symbolic-ref -q HEAD
```

Two facts, not one: the commit HEAD resolves to, and whether it is attached. A
detached HEAD resolving to a real commit differs from an attached HEAD at the same
commit, and only the second fact distinguishes them.

**Verified on Windows against a real `git`, and there are three states, not two:**

| State | `rev-parse HEAD` | `symbolic-ref -q HEAD` |
| --- | --- | --- |
| Attached, has commits | exit 0, the OID | exit 0, `refs/heads/<name>` |
| **Attached, unborn** | **exit 128**, `fatal: ambiguous argument 'HEAD'` | **exit 0**, names a branch that does not exist |
| Detached | exit 0, the OID | exit 1, no output |

The middle row is the trap. An unborn HEAD is *attached* -- `symbolic-ref` succeeds
and prints `refs/heads/main` -- so "symbolic-ref exit 0" does not mean the branch
exists, and a HEAD-state comparison that checks only the symbolic-ref exit code will
call an empty repository identical to a normal one. The comparison therefore records
all three facts together: the `rev-parse` exit code, its output, and the
`symbolic-ref` exit code and output. A missing exit code is not "the branch is
missing" in a way the caller has to guess at.

A `symbolic-ref` exit 1 is the normal answer for detached, carried as data
(FR-021), not as a failure.

### 5.3 Index -- exact

```
git ls-files -s
```

`-s` emits the stage number, so a conflicted index (stages 1, 2, 3) is distinguishable
from a clean one. For the Article V conflict state this is the observable that
matters.

### 5.4 Working tree -- exact, plus hashes

```
git status --porcelain=v2 --untracked-files=all
```

Then every path the index does not track is content-hashed and the digests compared
sorted. `status` alone reports *that* a file is modified; the hash proves *how* it
differs. Hashing bytes is not a Git object model (FR-024) -- it is comparing two
trees of files.

### 5.5 Reflog -- deliberately not raw

```
git reflog --format="%H %gs"
```

**Not** `git reflog` bare. A raw reflog line carries a Unix timestamp, and the two
sides of a differential run necessarily execute at different moments, so raw
comparison fails on a difference that is not a difference. `%H %gs` compares the
commit and the message, which are the parts that must match.

This is the same normalization principle as FR-008: exact where exactness is
meaningful, content-compared where an artifact of the test would otherwise dominate.

---

## 6. Fixture construction

Harness-only. These commands build the six baseline shapes; they are not part of the
product surface.

### Pinning (FR-011), applied to every fixture

```
git init -b main
git config user.name       "Gitflowfy Fixture"
git config user.email      "fixture@gitflowfy.invalid"
git config commit.gpgsign  false
git config tag.gpgsign     false
git config core.autocrlf   false
git config core.eol        lf
git config core.ignorecase false
git config init.defaultBranch main
```

Environment, per command:

```
GIT_AUTHOR_NAME      Gitflowfy Fixture
GIT_AUTHOR_EMAIL     fixture@gitflowfy.invalid
GIT_AUTHOR_DATE      2020-01-01T00:00:00+00:00
GIT_COMMITTER_NAME   Gitflowfy Fixture
GIT_COMMITTER_EMAIL  fixture@gitflowfy.invalid
GIT_COMMITTER_DATE   2020-01-01T00:00:00+00:00
TZ                   UTC
LC_ALL               C
LANG                 C
```

Every commit must carry a **distinct** author date, or two commits with identical
content and identical metadata would collapse to one OID and the fixture would have
fewer commits than it claims. The builder increments the timestamp per commit.

`core.ignorecase false` and `autocrlf false` are the two Windows-specific landmines;
without them a fixture built on Windows has different OIDs from the same fixture built
on Linux, which is risk R1.

### The six shapes

**Linear** -- `commit --allow-empty` x3 on `main`. Assert: one branch, no merge
commits, every commit has exactly one parent.

**Merged** -- `branch feature`, `commit` on it, `checkout main`,
`merge --no-ff feature`. Assert: one merge commit, exactly two parents, both
reachable.

**Octopus** -- `branch b2`, `branch b3`, an empty commit each,
`merge --no-ff b2 b3`. Assert: a merge commit with three or more parents, all
reachable.

**Orphan** -- `checkout --orphan island`, `commit --allow-empty`. Assert: no merge
base between `main` and `island` (`merge-base` exits 1).

**Detached** -- `checkout --detach main`. Assert: `symbolic-ref -q HEAD` exits 1 and
`rev-parse HEAD` succeeds.

**Empty** -- `init` and nothing else. Assert: `rev-parse HEAD` exits 128 with Git's
own unborn-HEAD error, *and* `symbolic-ref -q HEAD` exits 0 naming a branch that does
not exist. Both facts together are the empty-repository signature (see 5.2). Git's own
error text, verbatim, is the expected result (FR-026) -- not a failure of the tool.

### Each shape is built twice (FR-011, SC-009)

Two directories, same specification, compare OID lists. Gate G3 fails if they differ.

---

## 7. CI enforcement scripts

### G4: no git spawn outside the core

Searches every source file outside `crates/core` for:

```
Command::new("git")
process.spawn(
exec("git
```

Fails the build on any hit. Also covers the future `crates/ui` and `apps/desktop`,
which is the Article IV rule that the GUI never shells out.

### G5: no embedded Git implementation

```
cargo tree --all-features | grep -Ei 'libgit2|git2|gix|jgit'
```

Empty result required. This is the enforcement clause Article I names for itself.

### Scope check for NFR-001

Asserts the graph and renderer modules are untouched by this feature, so "this
feature claims no performance number" is a checked statement rather than a promise.
