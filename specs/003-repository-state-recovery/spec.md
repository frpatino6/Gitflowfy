---
description: Feature 003 - Repository state detection, display, and recovery affordances for all Article V unusual states. Full affordances for the 6 deferred states from Feature 001.
---

# Feature Specification: Repository State Recovery

**Feature Branch**: `003-repository-state-recovery`
**Status**: Draft
**Created**: 2026-09-30
**Constitution**: `.claude/constitution.md` v1.1.0

## Purpose

Feature 001 detect 6 estados Article V (fixtures) y diferenci 6 estados con obligacin estrecha: "return git's exit code, record it, do not hang". Feature 003 completa la promesa: **deteccin completa, display honesto, y affordances de recuperacin nombrando el comando exacto de Git**.

Estados cubiertos (Article V tabla):
1. Rebase en progreso
2. Merge conflict
3. Cherry-pick en progreso
4. Revert en progreso
5. Bisect en progreso
6. Partial clone
7. Sparse checkout
8. Missing reflog
9. Reftable backend
10. Submodule conflicts

## Scope

### In Scope

- Deteccin de los 10 estados Article V (deteccin via filesystem + `git status` + config)
- Display honesto: nombre del estado, archivo(s) involucrados, comando exacto de recuperacin
- Affordances de recuperacin: botones/acciones que ejecutan el comando exacto de Git
- Integracin con Feature 001: usa `GitCommand`, `AuditLog`, fixtures
- Integracin con Feature 002: grafo muestra estado visualmente (rama desconectada, commit conflictivo, etc.)

### Out of Scope

- Resolucin automtica de conflictos (usuario decide)
- UI compleja de resolucin merge (feature posterior)
- Testing en repos masivos (ya cubierto en Feature 001/002)

## User Stories

### Story 1: Detectar y Mostrar Estado (P0)

**As a** usuario abriendo un repo en estado inusual
**I want** ver claramente qu estado tiene mi repo
**so that** entienda qu pas y qu opciones tengo

**Independent test**: Para cada uno de los 10 estados, crear fixture, abrir en app, assert display muestra: nombre estado, archivos afectados, comando recuperacin.

**Acceptance scenarios**:
1. Given repo en rebase, when open app, then muestra "Rebase en progreso", archivo `.git/rebase-merge/`, botn "git rebase --continue / --abort"
2. Given merge conflict, when open app, then muestra "Merge conflict", lista archivos conflictivos, botones "git merge --continue / --abort"
3. Given cherry-pick conflict, when open app, then muestra "Cherry-pick en progreso", archivo conflictivo, botones "git cherry-pick --continue / --abort"
4. Given bisect, when open app, then muestra "Bisect en progreso", commit actual, botones "git bisect good/bad/skip/reset"
5. Given partial clone, when open app, then muestra "Partial clone", promisor remote, botn "git fetch --unshallow"
6. Given sparse checkout, when open app, then muestra "Sparse checkout", patrn actual, botn "git sparse-checkout disable"
7. Given missing reflog, when open app, then muestra "Reflog faltante", advertencia, botn "git reflog expire --expire=now --all" (con confirmacin)
8. Given reftable, when open app, then muestra "Reftable backend", versin Git, info only
9. Given submodule conflict, when open app, then muestra "Submodule conflict", path submdulo, botones "git submodule update --init / --remote"

### Story 2: Recuperacin Un-Click (P0)

**As a** usuario en estado inusual
**I want** ejecutar recuperacin con un click
**so that** no tenga que memorizar comandos Git

**Independent test**: Para cada estado, click botn recuperacin, assert comando ejecutado via `GitCommand`, estado resuelto, audit log registra invocacin.

**Acceptance scenarios**:
1. Given rebase, click "git rebase --continue", then ejecuta `git rebase --continue`, audit registra, repo sale de rebase
2. Given merge conflict resuelto manualmente, click "git merge --continue", then ejecuta `git merge --continue`, audit registra
3. Given cherry-pick abortado, click "git cherry-pick --abort", then ejecuta `git cherry-pick --abort`, repo limpio
4. Given bisect terminado, click "git bisect reset", then ejecuta `git bisect reset`, repo en branch original

### Story 3: Grafo Muestra Estado (P1)

**As a** usuario viendo grafo
**I want** ver indicadores visuales de estado inusual
**so that** entienda topologa sin leer texto

**Independent test**: Cargar cada fixture en Feature 002 renderer, assert marcadores visuales: commit conflictivo rojo, rama desconectada (orphan) punteada, HEAD detached icono, rebase en progreso badge.

**Acceptance scenarios**:
1. Given merge conflict, when graph rendered, then commit conflictivo marcado rojo, archivos listados en tooltip
2. Given orphan branches, when graph rendered, then componentes desconectados con estilo punteado
3. Given detached HEAD, when graph rendered, then HEAD muestra icono "detached"
4. Given rebase en progreso, when graph rendered, then badge "REBASE" en commits recientes

## Non-Functional Requirements

### NFR-001: Deteccin < 100ms

Deteccin de estado (filesystem + `git status` + config) completa en < 100ms en repo 600k commits.

### NFR-002: Sin Estado Paralelo

Deteccin usa solo filesystem + `git status` + config. No mantiene modelo paralelo. Cumple Article II.

### NFR-003: Recuperacin via Feature 001

Todas las recuperaciones usan `GitCommand` de Feature 001. Audit log registra cada invocacin. Cumple Article I, IV.

### NFR-004: Cross-Platform

Deteccin usa `git status --porcelain=v2` + filesystem portable. Windows + Linux CI.

### NFR-005: Integracin Graph (Feature 002)

Marcadores visuales en renderer reutilizan deteccin core. Sin duplicacin lgica.

## Constraints & Decisions

### Q1: UI Framework para Recovery Affordances -> **Resuelto: TypeScript en crates/ui (A)**

**Decisin**: Consistente con Feature 002 renderer. Webview ya cargada, mismo stack TypeScript/Canvas.

**Justificacin**: Reutiliza `crates/ui` stack, Tauri webview ya cargada, zero contexto extra. Rust native dialogs (B) requieren contexto extra.

### Q2: Conflict Resolution UI -> **Resuelto: Solo botones continue/abort (A)**

**Decisin**: Usuario resuelve en su editor. Tool solo ejecuta `git merge --continue / --abort` y `git cherry-pick --continue / --abort`.

**Justificacin**: Minimal scope. Editor externo ya tiene mejor diff tooling. Inline editor (B) es scope creep. External tool (C) aade dependencia.

### Q3: Bisect Visualization -> **Resuelto: Botones + grafo highlight (B)**

**Decisin**: Botones good/bad/skip/reset + grafo Feature 002 destaca commit actual + range bisect.

**Justificacin**: Reusa Feature 002 graph (ya validado). Aade valor visual sin componente nuevo. (A) muy minimal, (C) scope creep.

### Q4: Partial Clone / Sparse Checkout UI -> **Resuelto: Info + botn (A)**

**Decisin**: Muestra info (promisor remote / patrn sparse) + botn `git fetch --unshallow` / `git sparse-checkout disable`.

**Justificacin**: Fetch con progreso (B) es feature posterior (long-running operation UI). Config inline (C) scope creep. Botn simple ejecuta `GitCommand`, audit registra.

### Q5: Submodule Conflict Resolution -> **Resuelto: Botn `submodule update --init --recursive` (B)**

**Decisin**: Botn nico ejecuta `git submodule update --init --recursive`. Si hay conflictos en submodule, abre issue en UI (feature posterior).

**Justificacin**: `--recursive` cubre nested submodules. Nueva ventana (C) scope creep. Simple y completo.

---

## Clarification Check

- [x] Q1 resuelto
- [x] Q2 resuelto
- [x] Q3 resuelto
- [x] Q4 resuelto
- [x] Q5 resuelto

---

*Spec version 1.0.0 - all clarifications resolved, ready for review*