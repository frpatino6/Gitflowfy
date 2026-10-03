# Repository State Detection: Literal Commands & Observables

Feature 003 detecta 10 estados Article V + Clean. Todas las detecciones usan `git` real + filesystem.

## Detection Operations

| State | Detection Method | Command / Check |
| --- | --- | --- |
| RebaseInProgress | Filesystem + `git status` | `git rev-parse --git-dir` -> check `.git/rebase-merge/` o `.git/rebase-apply/` |
| MergeConflict | `git status --porcelain=v2` | `git status --porcelain=v2` -> lines starting with `u` (unmerged) |
| CherryPickInProgress | Filesystem + `git status` | `.git/CHERRY_PICK_HEAD` exists |
| RevertInProgress | Filesystem + `git status` | `.git/REVERT_HEAD` exists |
| BisectInProgress | Filesystem + `git bisect log` | `.git/BISECT_LOG` exists |
| PartialClone | Git config | `git config --get remote.<name>.promisor` = `true` |
| SparseCheckout | Git config | `git config --get core.sparseCheckout` = `true` |
| MissingReflog | Filesystem | `.git/logs/` missing or empty |
| ReftableBackend | Git config | `git config --get extensions.refStorage` = `reftable` |
| SubmoduleConflict | `git submodule status` | `git submodule status` -> lines starting with `+` or `-` |

## Recovery Operations (Literal Commands)

| Action | Command | Exit Codes |
| --- | --- | --- |
| Rebase Continue | `git rebase --continue` | 0 success, 1 conflict, 128 error |
| Rebase Abort | `git rebase --abort` | 0 success |
| Merge Continue | `git merge --continue` | 0 success, 1 conflict |
| Merge Abort | `git merge --abort` | 0 success |
| CherryPick Continue | `git cherry-pick --continue` | 0 success, 1 conflict |
| CherryPick Abort | `git cherry-pick --abort` | 0 success |
| Revert Continue | `git revert --continue` | 0 success, 1 conflict |
| Revert Abort | `git revert --abort` | 0 success |
| Bisect Good | `git bisect good <rev>` | 0 success |
| Bisect Bad | `git bisect bad <rev>` | 0 success |
| Bisect Skip | `git bisect skip <rev>` | 0 success |
| Bisect Reset | `git bisect reset` | 0 success |
| Unshallow | `git fetch --unshallow` | 0 success |
| Sparse Checkout Disable | `git sparse-checkout disable` | 0 success |
| Submodule Update Recursive | `git submodule update --init --recursive` | 0 success |

## Detection Algorithm (Pseudocode)

```rust
fn detect_state(repo: &Path) -> RepoState {
    // Priority order: rebase > merge > cherry-pick > revert > bisect > partial > sparse > reflog > reftable > submodule > clean
    
    let git_dir = git_rev_parse_git_dir(repo)?;
    
    // 1. Rebase
    if git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists() {
        return RebaseInProgress { ... };
    }
    
    // 2. Merge conflict
    let status = git_status_porcelain_v2(repo)?;
    if status.lines().any(|l| l.starts_with('u')) {
        return MergeConflict { files: parse_unmerged(&status) };
    }
    
    // 3. Cherry-pick
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return CherryPickInProgress { ... };
    }
    
    // 4. Revert
    if git_dir.join("REVERT_HEAD").exists() {
        return RevertInProgress { ... };
    }
    
    // 4. Bisect
    if git_dir.join("BISECT_LOG").exists() {
        return BisectInProgress { ... };
    }
    
    // 5. Partial clone
    if git_config_bool("remote.*.promisor")? {
        return PartialClone { ... };
    }
    
    // 6. Sparse checkout
    if git_config_bool("core.sparseCheckout")? {
        return SparseCheckout { ... };
    }
    
    // 5. Missing reflog
    if !repo.join(".git/logs").exists() {
        return MissingReflog;
    }
    
    // 6. Reftable
    if git_config("extensions.refStorage")? == "reftable" {
        return ReftableBackend;
    }
    
    // 7. Submodule conflict
    let sub_status = git_submodule_status(repo)?;
    if sub_status.lines().any(|l| l.starts_with('+') || l.starts_with('-')) {
        return SubmoduleConflict { ... };
    }
    
    Clean
}
```

## Recovery Commands (via GitCommand)

```rust
fn recover_action(repo: &Path, action: RecoveryAction) -> Result<Invocation> {
    let cmd = match action {
        RecoveryAction::RebaseContinue => GitCommand::new(repo, ["rebase", "--continue"]),
        RecoveryAction::RebaseAbort => GitCommand::new(repo, ["rebase", "--abort"]),
        RecoveryAction::MergeContinue => GitCommand::new(repo, ["merge", "--continue"]),
        RecoveryAction::MergeAbort => GitCommand::new(repo, ["merge", "--abort"]),
        RecoveryAction::CherryPickContinue => GitCommand::new(repo, ["cherry-pick", "--continue"]),
        RecoveryAction::CherryPickAbort => GitCommand::new(repo, ["cherry-pick", "--abort"]),
        RecoveryAction::RevertContinue => GitCommand::new(repo, ["revert", "--continue"]),
        RecoveryAction::RevertAbort => GitCommand::new(repo, ["revert", "--abort"]),
        RecoveryAction::BisectGood => GitCommand::new(repo, ["bisect", "good"]),
        RecoveryAction::BisectBad => GitCommand::new(repo, ["bisect", "bad"]),
        RecoveryAction::BisectSkip => GitCommand::new(repo, ["bisect", "skip"]),
        RecoveryAction::BisectReset => GitCommand::new(repo, ["bisect", "reset"]),
        RecoveryAction::Unshallow => GitCommand::new(repo, ["fetch", "--unshallow"]),
        RecoveryAction::SparseDisable => GitCommand::new(repo, ["sparse-checkout", "disable"]),
        RecoveryAction::SubmoduleUpdateRecursive => GitCommand::new(repo, ["submodule", "update", "--init", "--recursive"]),
        RecoveryAction::WarnOnly => return Err("No action available"),
        RecoveryAction::InfoOnly => return Err("No action available"),
    };
    cmd.run()
}
```

## UI Affordances (TypeScript)

```typescript
// affordances.ts
interface Affordance {
    label: string;
    action: RecoveryAction;
    primary: boolean;
    danger: boolean;
}

const AFFORDANCES: Record<RepoState, Affordance[]> = {
    RebaseInProgress: [
        { label: "Continuar rebase", action: "RebaseContinue", primary: true, danger: false },
        { label: "Abortar rebase", action: "RebaseAbort", primary: false, danger: true },
    ],
    MergeConflict: [
        { label: "Continuar merge", action: "MergeContinue", primary: true, danger: false },
        { label: "Abortar merge", action: "MergeAbort", primary: false, danger: true },
    ],
    CherryPickInProgress: [
        { label: "Continuar cherry-pick", action: "CherryPickContinue", primary: true, danger: false },
        { label: "Abortar cherry-pick", action: "CherryPickAbort", primary: false, danger: true },
    ],
    RevertInProgress: [
        { label: "Continuar revert", action: "RevertContinue", primary: true, danger: false },
        { label: "Abortar revert", action: "RevertAbort", primary: false, danger: true },
    ],
    BisectInProgress: [
        { label: "Bueno", action: "BisectGood", primary: true, danger: false },
        { label: "Malo", action: "BisectBad", primary: true, danger: false },
        { label: "Saltar", action: "BisectSkip", primary: false, danger: false },
        { label: "Reset bisect", action: "BisectReset", primary: false, danger: true },
    ],
    PartialClone: [
        { label: "Fetch completo (--unshallow)", action: "Unshallow", primary: true, danger: false },
    ],
    SparseCheckout: [
        { label: "Desactivar sparse checkout", action: "SparseDisable", primary: true, danger: false },
    ],
    SubmoduleConflict: [
        { label: "Actualizar submdulos (recursivo)", action: "SubmoduleUpdateRecursive", primary: true, danger: false },
    ],
    MissingReflog: [], // Solo warning
    ReftableBackend: [], // Solo info
    Clean: [], // Sin affordances
};
```

## Graph Markers (Feature 002 Integration)

```typescript
// markers.ts
interface GraphMarker {
    commitOid: string;
    type: 'conflict' | 'detached' | 'orphan' | 'rebasing' | 'bisect' | 'submodule';
    label: string;
    color: string;
}

function getMarkersForState(state: RepoState, repo: string): GraphMarker[] {
    switch (state) {
        case MergeConflict:
            return state.files.map(f => ({ commitOid: getCommitForFile(f), type: 'conflict', color: '#ff4444' }));
        case Detached: // from Feature 001
            return [{ commitOid: getHeadOid(repo), type: 'detached', label: 'HEAD detached', color: '#ffaa00' }];
        case RebaseInProgress:
            return [{ commitOid: getCurrentRebaseCommit(), type: 'rebasing', label: 'Rebasing...', color: '#00aaff' }];
        case BisectInProgress:
            return [{ commitOid: getCurrentBisectCommit(), type: 'bisect', label: 'Bisecting...', color: '#aa00ff' }];
        case SubmoduleConflict:
            return [{ commitOid: getSubmoduleHead(), type: 'submodule', color: '#ff8800' }];
        case _ => [];
    }
}
```

## Tests

| Test | Description |
| --- | --- |
| `t_detect_each_state` | 10 fixtures, cada una detecta estado correcto |
| `t_detect_priority` | Estados compuestos resueltos por prioridad (rebase > merge > ...) |
| `t_detect_perf` | 600k repo, deteccin < 100ms |
| `t_recover_each_action` | 13 acciones, cada una invoca comando correcto |
| `t_recover_audit_records` | Cada recover registra en AuditLog |
| `t_affordances_match_state` | Cada estado devuelve affordances correctas |
| `t_graph_markers` | 10 estados + clean, marcadores correctos |

---

*Implementation details version 1.0.0 - alineado con spec v1.0.0*