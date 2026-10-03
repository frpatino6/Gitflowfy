# Research: Repository State Recovery

La evidencia detrs de las decisiones en `plan.md`.

## Pregunta

Cmo detectar y recuperar 10 estados Article V sin mantener modelo paralelo (Article II).

## Lo que Article V exige

| Estado | Detectado via | Recuperacin |
| --- | --- | --- |
| Rebase in progress | `.git/rebase-merge/` o `.git/rebase-apply/` | `git rebase --continue/--abort` |
| Merge conflict | `git status --porcelain=v2` (unmerged) | `git merge --continue/--abort` |
| Cherry-pick in progress | `CHERRY_PICK_HEAD` | `git cherry-pick --continue/--abort` |
| Revert in progress | `REVERT_HEAD` | `git revert --continue/--abort` |
| Bisect in progress | `BISECT_LOG` | `git bisect good/bad/skip/reset` |
| Partial clone | `remote.*.promisor` | `git fetch --unshallow` |
| Sparse checkout | `core.sparseCheckout` | `git sparse-checkout disable` |
| Missing reflog | `.git/logs/` missing | Warn only |
| Reftable backend | `extensions.refStorage` | Info only |
| Submodule conflict | `git submodule status` (+/-) | `git submodule update --init --recursive` |

## Decisiones de diseo

### 1. Deteccin via filesystem + `git status` + config (no modelo paralelo)

Article II prohbe modelo paralelo del repo. Deteccin usa solo:
- Filesystem checks (`.git/rebase-merge/`, `CHERRY_PICK_HEAD`, etc.)
- `git status --porcelain=v2` (unmerged entries)
- Git config queries (`git config --get`)
- `git submodule status`

**Nada de esto mantiene estado** - solo lee lo que Git ya escribe.

### 2. Prioridad de estados compuestos

Un repo puede estar en mltiples estados (ej: rebase + merge conflict si rebase pausado en conflict). Prioridad:

1. Rebase in progress (ms especfico)
2. Merge conflict
3. Cherry-pick in progress
4. Revert in progress
5. Bisect in progress
6. Partial clone
7. Sparse checkout
8. Missing reflog
9. Reftable backend
10. Submodule conflict
11. Clean

Esto evita ambigedad y matcha comportamiento Git real.

### 3. Recuperacin via Feature 001 `GitCommand`

Todas las acciones de recuperacin usan `GitCommand` de Feature 001:
- Single spawn site
- Audit log automtico
- Refusal de network ops
- Concurrency control per repo

No hay cdigo de recuperacin separado - solo construye `GitCommand` con args correctos.

### 4. UI Affordances como data, no cdigo

`AFFORDANCES` map en TypeScript es data pura. Renderer itera y renderiza botones. Aadir estado = aadir entrada al map, no cdigo nuevo.

### 5. Graph markers reusan Feature 002

`markers.ts` convierte `RepoState` a marcadores visuales. Reusa `crates/ui` renderer. Sin duplicacin.

## Performance

| Operacin | Target | Mtodo |
| --- | --- | --- |
| Detect state (600k) | < 100ms | `git status --porcelain=v2` + filesystem checks (paralelo) |
| Recover action | < 5ms overhead | Feature 001 G2 budget |
| UI render affordances | < 16ms | React-like virtual DOM en `crates/ui` |

## Integracin con Features previas

| Feature | Integracin |
| --- | --- |
| 001 Headless | `GitCommand`, `AuditLog`, `RepoLock`, fixtures |
| 002 Graph | `markers.ts` -> graph markers, `affordances.ts` -> UI |
| 003 State | Core detection + UI affordances + graph markers |

## Decisiones descartadas

| Opcin | Por qu no |
| --- | --- |
| Modelo paralelo de estado | Viola Article II |
| Parsing `.git` directo sin `git` command | Frgil, rompe con Git updates |
| Inline conflict editor | Scope creep, Article VIII |
| Auto-recovery sin confirmacin | Peligroso, usuario debe decidir |

---

*Research version 1.0.0*