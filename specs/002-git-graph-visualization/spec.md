---
description: Feature 002 - a Git commit graph visualization with Canvas renderer, CSR layout, and Tauri v2 integration. Meets Article VI budgets.
---

# Feature Specification: Git Graph Visualization

**Feature Branch**: `002-git-graph-visualization`
**Status**: Draft
**Created**: 2026-09-30
**Constitution**: `.claude/constitution.md` v1.1.0

## Purpose

Feature 001 delivered the headless Git operation surface: spawn, audit, fixtures, differential harness. It did not include the commit graph or renderer. The spike in `spike/` proved the graph architecture works: 599,555 commits, 9.6 MB graph, 522 ms renderer load, p99 19.5 ms frame time.

This feature ports the spike's graph builder and renderer into the product:

1. **Graph builder**: streams `git log`, builds CSR layout in typed arrays, writes `graph.bin`
2. **Renderer**: Canvas with typed arrays, 60 fps p50, 30 fps p99, 50 ms max frame
3. **Tauri v2 integration**: webview hosts renderer, core sends `graph.bin` via command surface

The graph is the core differentiator: incumbents (GitKraken, Fork, Extensions) reimplement Git's operation layer to support their own undo, which limits command coverage and breaks on unusual repo states. Our graph is built from real `git` output via the headless surface, so it reflects *exactly* what Git sees — including unusual states from Feature 001's fixtures.

## Scope

### In Scope

- Graph builder: `git log --all --topo-order --format=...` → CSR typed arrays → `graph.bin`
- Renderer: Canvas 2D, typed arrays, virtualized viewport, 60 fps p50
- Tauri v2: webview hosts renderer, core sends `graph.bin` via command surface
- Binary format: CSR layout (5 typed arrays) carried byte-for-byte from spike
- Article VI budgets: extract < 30s, graph < 32 MB, load < 2s, frame p50 16.7ms, p99 33.4ms, max 50ms
- Integration with Feature 001: graph built via `GitCommand` (not direct spawn), fixtures reused

### Out of Scope

- GUI chrome (toolbars, menus, sidebars) — separate feature
- User interactions beyond viewport pan/zoom — separate feature
- Multi-repo tabs — separate feature
- Search/filter UI — separate feature

## User Stories

### Story 1: Extract and Build Graph (P0)

**As a** user opening a repository
**I want** the commit graph built from my Git history
**so that** I can see the complete topology instantly

**Independent test**: Run graph builder on llvm/llvm-project (599,555 commits). Assert binary size < 32 MB, extract time < 30 s, OIDs match `git rev-list`.

**Acceptance scenarios**:
1. Given a repo with 600k commits, when build runs, then `graph.bin` produced in < 30 s
2. Given the binary, when loaded, then all 5 arrays have correct lengths and OIDs match `git rev-list --all`
3. Given a repo with octopus merges, when built, then parents array correctly represents all parents

### Story 2: Render Graph at 60 fps (P0)

**As a** user viewing the graph
**I want** smooth pan/zoom at 60 fps
**so that** exploration feels instantaneous

**Independent test**: Load 600k-commit graph, pan 1000px horizontally, record frame times. Assert p50 ≤ 16.7 ms, p99 ≤ 33.4 ms, max ≤ 50 ms.

**Acceptance scenarios**:
1. Given loaded graph, when panning, then frame p50 ≤ 16.7 ms, p99 ≤ 33.4 ms
2. Given loaded graph, when zooming, then no frame > 50 ms
3. Given 600k commits, when initial render, then load < 2 s

### Story 3: Tauri Integration (P0)

**As a** user opening the app
**I want** the graph visible in a native window
**so that** I can use it like any desktop app

**Independent test**: Launch Tauri app, load 600k repo, verify webview receives `graph.bin` and renders.

**Acceptance scenarios**:
1. Given Tauri app launched, when repo selected, then webview loads and renders graph
2. Given graph rendered, when pan/zoom, then frame budgets met
3. Given Feature 001 headless surface, when graph requested, then core returns `graph.bin` bytes

### Story 4: Unusual States Visible (P1)

**As a** user with a repo in an unusual state
**I want** the graph to show the true topology
**so that** I can recover correctly

**Independent test**: Run graph builder on each Feature 001 fixture (linear, merged, octopus, orphan, detached, empty). Assert OIDs match and topology matches differential harness truth.

**Acceptance scenarios**:
1. Given empty repo, when graph built, then renders empty state (no crash)
2. Given detached HEAD, when graph built, then shows detached commit correctly
3. Given orphan branches, when graph built, then shows disconnected components

## Non-Functional Requirements

### NFR-001: Article VI Budgets

| Metric | Budget | Measured (spike) |
|--------|--------|------------------|
| Extract 600k commits | < 30 s | 9.7 s |
| In-memory graph size | < 32 MB | 9.6 MB |
| Load graph into renderer | < 2 s | 522 ms |
| Frame p50 (pan/zoom) | < 16.7 ms (60 fps) | 16.7 ms |
| Frame p99 (pan/zoom) | < 33.4 ms (30 fps) | 19.5 ms |
| Worst single frame | < 50 ms | 20.2 ms |

These are **contracts**, not aspirations. CI fails on regression.

### NFR-002: Binary Format Stability

The `graph.bin` format (5 typed arrays, CSR layout) is the contract between builder and renderer. Changes require:
1. Version bump in binary header
2. Migration path for existing binaries
3. Renderer handles both versions

### NFR-003: Tauri Command Surface

Graph operations exposed via Feature 001's command surface:
- `gitflowfy graph build <repo> <output>` — builds `graph.bin`
- `gitflowfy graph load <repo>` — returns bytes for webview
- No direct renderer access from CLI

### NFR-004: Cross-Platform

Renderer: Canvas 2D (works Windows/Linux/macOS via webview)
Builder: Rust (compiles everywhere)
CI: Windows + Linux required

### NFR-005: No GUI in Core

`crates/core` remains headless. Renderer lives in `crates/ui` (web code), Tauri in `apps/desktop`.

## Constraints & Decisions

### Q1: Renderer Language → **Resolved: TypeScript in Tauri webview (A)**

**Decision**: Reuse spike renderer directly in Tauri webview. Zero port effort, validated budgets.

**Rationale**: Spike renderer already validated at 19.5 ms p99. Porting to Rust (B)/(C) adds effort with no proven benefit. Article VI budgets already met by (A).

### Q2: Graph Incremental Updates → **Resolved: Full rebuild (A)**

**Decision**: Full rebuild on every change. 9.7 s is acceptable for initial implementation.

**Rationale**: Simpler, no correctness risk. Incremental (B) requires parent index updates and risks OID drift. Hybrid (C) adds complexity without proven need. Revisit if users report latency complaints.

### [NEEDS CLARIFICATION] Q3: Large File Handling

### Q3: Large File Handling → **Resolved: Tauri invoke with ArrayBuffer (A)**

**Decision**: Pass `graph.bin` as `ArrayBuffer` via Tauri `invoke`. Zero-copy, 9.6 MB is small.

**Rationale**: Fastest, zero-copy. Base64 (B) adds 33% overhead. HTTP endpoint (C) adds complexity for no benefit at 9.6 MB.

---

### Q4: Graph Layout Algorithm → **Resolved: Port spike's lane assignment exactly (A)**

**Decision**: Port the spike's lane assignment algorithm exactly. Guarantees spike parity.

**Rationale**: Spike already validated at 19.5 ms p99. Improving (B) risks budget regression. External lib (C) violates Article VIII.

### Q5: Webview Communication → **Resolved: Tauri invoke with ArrayBuffer (A)**

**Decision**: Pass `graph.bin` as `ArrayBuffer` via Tauri `invoke`. Fastest, zero-copy.

**Rationale**: Direct, typed, zero-copy. Base64 (B) adds overhead. HTTP endpoint (C) adds latency and complexity.

---

## Clarification Check

- [x] Q1 resolved
- [x] Q2 resolved
- [x] Q3 resolved
- [x] Q4 resolved
- [x] Q5 resolved

---

*Spec version 1.0.0 — all clarifications resolved, ready for review*