# Implementation Plan: Repository State Recovery

**Branch**: `003-repository-state-recovery`
**Spec**: [spec.md](spec.md)
**Status**: Draft
**Created**: 2026-09-30

---

## Phase -1: Pre-Implementation Gates

### Constitution Check

| Article | Binding here? | How this plan satisfies it |
| --- | --- | --- |
| I - Shell out | **S** | Todas las recuperaciones usan `GitCommand` de Feature 001. No spawn directo. |
| II - Reflog only | **S** | Deteccin usa solo filesystem + `git status` + config. No modelo paralelo. |
| III - Test first | **S** | Phase 0 tests detection antes que UI. |
| IV - CLI first | **S** | `gitflowfy state detect <repo>` y `gitflowfy state recover <repo> <action>` sin ventana. |
| V - Odd repo states | **S, crtico** | Esta feature **es** Article V. 10 estados detectados + affordances. |
| VI - Perf budgets | **S** | NFR-001: deteccin < 100ms. Gate G2 mide. |
| VII - <=3 crates | **S** | Aade lgica a `crates/core` + UI en `crates/ui`. Total = 3/3. |
| VIII - Anti-abstraction | **S** | Deteccin usa `GitCommand` + filesystem. Sin traits/wrappers. |
| IX - Integration first | **S** | Fixtures reales de Feature 001 (10 estados). Tests contra `git` real. |

### Gate G1: git usable
**Pass**: `git version` parseable.

### Gate G2: Deteccin < 100ms
**Pass**: Medir deteccin (filesystem + `git status` + config) en repo 600k commits.

### Gate G3: Fixtures OIDs reproducibles
**Pass**: Reutiliza Feature 001 fixtures (ya validado G3).

### Gate G4: No git spawn fuera de core
**Pass**: Script existente.

### Gate G5: No embedded Git
**Pass**: Script existente.

---

## Technical Approach

### Stack Decision

**Elegido**: 
- **Core detection**: Rust en `crates/core/src/state/` (usa `GitCommand` + filesystem)
- **UI affordances**: TypeScript en `crates/ui/src/state/` (botones + display, reusa `crates/ui` stack)
- **Desktop**: Ya existe en Feature 002 (`apps/desktop`)

### Architecture

```
crates/core/
  state/
    mod.rs
    detect.rs       # detect_state(repo) -> RepoState enum (10 variants)
    recover.rs      # recover_action(repo, action) -> GitCommand
    types.rs        # RepoState, RecoveryAction, StateInfo

crates/ui/
  src/
    state/
      detect.ts       # Reusa detect logic via invoke
      affordances.ts  # Botones + display por estado
      mod.ts          # Entry point

apps/desktop/
  # Ya existe (Feature 002)
```

### Data Flow

1. **Detect**: `gitflowfy state detect <repo>` -> `detect_state()` -> `RepoState` (enum 10 variants + `Clean`)
2. **Display**: UI llama `invoke("state:detect", { repo })` -> `RepoState` -> render affordances
3. **Recover**: Usuario click botn -> `invoke("state:recover", { repo, action })` -> `recover_action()` -> `GitCommand` -> audit log -> estado actualizado

### RepoState Enum (10 + Clean)

```rust
pub enum RepoState {
    Clean,
    RebaseInProgress { dir: PathBuf, step: usize, total: usize },
    MergeConflict { files: Vec<PathBuf> },
    CherryPickInProgress { commit: String, files: Vec<PathBuf> },
    RevertInProgress { commit: String, files: Vec<PathBuf> },
    BisectInProgress { start: String, bad: String, current: String, step: usize, total: usize },
    PartialClone { promisor: String },
    SparseCheckout { pattern: String },
    MissingReflog,
    ReftableBackend,
    SubmoduleConflict { path: PathBuf },
}
```

### Recovery Actions

| State | Actions | Git Command |
| --- | --- | --- |
| RebaseInProgress | `Continue`, `Abort` | `git rebase --continue`, `git rebase --abort` |
| MergeConflict | `Continue`, `Abort` | `git merge --continue`, `git merge --abort` |
| CherryPickInProgress | `Continue`, `Abort` | `git cherry-pick --continue`, `git cherry-pick --abort` |
| RevertInProgress | `Continue`, `Abort` | `git revert --continue`, `git revert --abort` |
| BisectInProgress | `Good`, `Bad`, `Skip`, `Reset` | `git bisect good/bad/skip/reset` |
| PartialClone | `Unshallow` | `git fetch --unshallow` |
| SparseCheckout | `Disable` | `git sparse-checkout disable` |
| MissingReflog | `WarnOnly` | (no action, solo warning) |
| ReftableBackend | `InfoOnly` | (info only) |
| SubmoduleConflict | `UpdateRecursive` | `git submodule update --init --recursive` |

---

## Requirement Traceability

| Requirement | Technical decision | Verified by |
| --- | --- | --- |
| Story 1: Detect + Display | `detect.rs` + `affordances.ts` | `t_detect_each_state`, `t_display_affordances` |
| Story 2: One-click Recover | `recover.rs` + `GitCommand` | `t_recover_each_action`, `t_audit_records_recovery` |
| Story 3: Graph markers | `crates/ui/src/state/markers.ts` | `t_graph_markers_each_state` |
| NFR-001: Detect < 100ms | `detect.rs` optimized | `t_detect_perf` |
| NFR-002: No parallel state | Enum only, no model | Compile-time check |
| NFR-003: Recover via Feature 001 | `recover.rs` usa `GitCommand` | `t_recover_uses_gitcommand` |
| NFR-004: Cross-platform | `git status --porcelain=v2` | CI Windows + Linux |
| NFR-005: Graph integration | `markers.ts` usa `RepoState` | `t_graph_markers_*`

---

## Performance Impact

| Metric | Budget | New expected | Measured |
| --- | --- | --- | --- |
| Detect state (600k repo) | < 100 ms | < 50 ms | Gate G2 + `t_detect_perf` |
| Recover action overhead | < 5 ms (Feature 001 G2) | < 5 ms | Feature 001 gate G2 |
| UI render affordances | < 16 ms | < 16 ms | Feature 002 frame budget |

---

## Project Structure

```
crates/core/
  src/
    state/
      mod.rs
      detect.rs       # detect_state()
      recover.rs      # recover_action()
      types.rs        # RepoState, RecoveryAction, StateInfo

crates/ui/
  src/
    state/
      detect.ts       # invoke("state:detect")
      affordances.ts  # Botones por RepoState
      markers.ts      # Graph markers por RepoState
      mod.ts

apps/desktop/
  # Existente
```

---

## Complexity Tracking

| Exception | Article | Justification | Revisit when |
| --- | --- | --- | --- |
| `serde` para `invoke` payload | VIII | Serializacin en boundary. No wrapper. | Si serde MSRV/license conflict. |
| `serde_json` para CLI JSON | VIII | Boundary nico. | Nunca. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| R1: Detect false positive/negative | Media | Usuario ve estado wrong | Tests exhaustivos 10 estados + `git status` truth. |
| R2: Recover action falla silenciosamente | Baja | Usuario cree que recuper | `GitCommand` propaga exit code/stderr. UI muestra error. |
| R3: Estado compuesto (rebase + merge) | Baja | Deteccin ambigua | Prioridad: rebase > merge > cherry-pick > revert > bisect. Tests. |
| R4: UI affordances no accesibles | Media | a11y fail | Botones semantic HTML, ARIA labels, keyboard nav. |

---

## Plan Self-Review

- [x] Every requirement traced to decision + test
- [x] All 5 clarifications resolved in spec
- [x] Every git command spelled out literally
- [x] Headless surface defined before GUI (Phase 1 before Phase 2)
- [x] Fixtures for all 10 states (reused from Feature 001 fixtures + new)
- [x] Perf budget stated + measurement (NFR-001 + gate G2)
- [x] Complexity exceptions justified (2, each with trigger)
- [x] Pure ASCII output

---

*Plan version 1.0.0 - listo para revisin constitucional*