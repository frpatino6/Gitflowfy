# Implementation Plan: Git Graph Visualization

**Branch**: `002-git-graph-visualization`
**Spec**: [spec.md](spec.md)
**Status**: Draft
**Created**: 2026-09-30

---

## Phase -1: Pre-Implementation Gates

> Gates para evitar over-engineering. Un gate fallido se resuelve o se documenta en Complexity Tracking.

### Constitution Check

| Article | Binding here? | How this plan satisfies it |
| --- | --- | --- |
| I - Shell out | **S** | Graph builder usa `GitCommand` de Feature 001 (nico spawn site). No `libgit2`/`gix`. CI gate G4/G5. |
| II - Reflog only | **S** | Graph es datos derivados de `git log` real. No estado paralelo. |
| III - Test first | **S** | Phase 0 construye fixtures/harness antes que builder/renderer. |
| IV - CLI first | **S** | `gitflowfy graph build/load` funciona sin ventana. Tauri es Phase 3. |
| V - Odd repo states | **S** | Fixtures de Feature 001 (6 estados) + deferred. Graph builder los maneja. |
| VI - Perf budgets | **S, crtico** | Article VI budgets son vinculantes. Spike ya valid: 9.7s, 9.6MB, 522ms, p99 19.5ms. CI re-mide. |
| VII - <=3 crates | **S** | Aade `crates/ui` (renderer web) + `apps/desktop` (Tauri). Total = 3/3. |
| VIII - Anti-abstraction | **S** | Renderer TypeScript directo en webview. Builder Rust usa `GitCommand`. Sin traits/wrappers. |
| IX - Integration first | **S** | Fixtures reales de Feature 001. Builder usa `git log` real. Renderer en webview real. |

- [x] Every applicable article satisfied
- [x] No `libgit2`/`gix`/`jgit` (gate G5)
- [x] No `git` spawn fuera de `crates/core` (gate G4)

### Gate G1: git usable
**Pass**: `git version` parseable, repo temporal creable.

### Gate G2: Article VI budgets alcanzables
**Pass**: Medir builder + renderer contra spike budgets (ya validados en spike). Re-medir en CI.

### Gate G3: Fixture OIDs reproducibles
**Pass**: Ya validado en Feature 001 (G3). Fixtures reutilizadas.

### Gate G4: No git spawn fuera de core
**Pass**: Script `check-no-git-outside-core` existente.

### Gate G5: No embedded Git
**Pass**: Script `check-no-embedded-git` existente.

### Simplicity Gate (Article VII)
- [x] Total crates = 3 (`core`, `ui`, `desktop`)
- [x] No crate extra

### Anti-Abstraction Gate (Article VIII)
- [x] No single-impl traits
- [x] No DI container
- [x] Renderer TypeScript directo, builder Rust usa `GitCommand`

### Integration-First Gate (Article IX)
- [x] Fixtures reales de Feature 001
- [x] Builder usa `git log` real via `GitCommand`
- [x] Renderer en webview real (Tauri)

### Gate Result
**PASS** - Ver Complexity Tracking para excepciones documentadas.

---

## Technical Approach

### Stack Decision

**Elegido**: 
- **Builder**: Rust en `crates/core` (reutiliza `GitCommand`, `graph.bin` idntico a spike)
- **Renderer**: TypeScript en `crates/ui` (Canvas 2D + typed arrays, puerto directo de `spike/`)
- **Desktop**: Tauri v2 en `apps/desktop` (webview hostea renderer)

**Razonamiento**:
- Spike validado: 599,555 commits, 9.7s, 9.6MB, p99 19.5ms
- Renderer puerto directo (0 esfuerzo, budgets garantizados)
- Builder Rust: reutiliza `GitCommand`, `graph.bin` byte-identico a spike
- Tauri v2: webview hostea renderer, IPC via `invoke` con `ArrayBuffer`

**Electrn descartado**: Chromium por instancia compite con 9.6MB graph + render loop.

### Architecture

```
crates/core/
  git/              # Feature 001 (existente)
  graph/
    builder.rs      # git log -> CSR typed arrays -> graph.bin (puerto spike/build-graph.mjs)
    format.rs       # graph.bin format v1 (5 typed arrays CSR)
  cli/              # Feature 001 + graph subcommands

crates/ui/
  src/
    renderer.ts     # Canvas 2D + typed arrays (puerto spike/public/index.html)
    viewport.ts     # Virtualized pan/zoom
    main.ts         # Entry point webview

apps/desktop/
  src/
    main.rs         # Tauri v2 setup, webview + invoke handlers
    tauri.conf.json # Config
```

### Data Flow

1. **Build**: `gitflowfy graph build <repo> <out>` -> `GitCommand("log --all --topo-order --format=...")` -> parse -> CSR typed arrays -> `graph.bin` (v1 header + 5 arrays)
2. **Load**: `gitflowfy graph load <repo>` -> devuelve `graph.bin` bytes (Tauri `invoke` -> `ArrayBuffer`)
3. **Render**: Webview carga `renderer.ts` -> `fetch`/`invoke` -> `graph.bin` -> decode typed arrays -> Canvas 2D virtualizado

### Graph Operations (Literal Commands)

| Operation | Command | Notes |
| --- | --- | --- |
| Extract graph | `git log --all --topo-order --format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s` | `x1f` = unit separator |
| Extract refs | `git for-each-ref --format="%(refname) %(objectname)"` | Para etiquetas ramas/tags |

### Binary Format (`graph.bin` v1)

| Offset | Type | Description |
| --- | --- | --- |
| 0-3 | u32 | Magic `0x47524150` ("GRAP") |
| 4 | u8 | Version (1) |
| 5-8 | u32 | Commit count (n) |
| 9-12 | u32 | Edge count (m) |
| 13- | `n*4` | `u32[]` parent_index (CSR offsets, len n+1) |
| 13+n*4 | `m*4` | `u32[]` parent_list (CSR edges) |
| ... | `n*8` | `i64[]` commit_time (author timestamp) |
| ... | `n*4` | `u32[]` lane (assigned lane 0..L-1) |
| ... | `n*4` | `u32[]` parent_count (0, 1, 2, 3+) |
| ... | variable | String pool: subjects + author names + refs (null-terminated) |

*Byte-for-byte compatible con `spike/build-graph.mjs` output.*

### Tauri Integration

| Command | Handler | Payload |
| --- | --- | --- |
| `graph:load` | `invoke("graph:load", { repo })` | Returns `ArrayBuffer` (`graph.bin`) |
| `graph:build` | CLI subcommand | `gitflowfy graph build <repo> <out>` |

Webview: `const buf = await invoke("graph:load", { repo });` -> `decodeGraph(new Uint8Array(buf))` -> render.

---

## Requirement Traceability

> Every requirement from spec maps to technical decision and test.

| Requirement | Technical decision | Verified by |
| --- | --- | --- |
| Story 1: Extract/build | `crates/core/src/graph/builder.rs` + `GitCommand` | `t_extract_within_budget`, `t_oids_match_git` |
| Story 2: Render 60fps | `crates/ui/src/renderer.ts` + `viewport.ts` | `t_frame_p50`, `t_frame_p99`, `t_load_time` |
| Story 3: Tauri | `apps/desktop` + `invoke("graph:load")` | `t_tauri_loads_graph`, `t_tauri_renders` |
| Story 4: Unusual states | Fixtures Feature 001 reutilizadas | `t_graph_fixture_*` (6 fixtures) |
| NFR-001: Article VI | Budgets en CI | `t_extract_<30s`, `t_graph_<32MB`, `t_load_<2s`, `t_frame_*` |
| NFR-002: Binary format | `graph.bin` v1 header + CSR | `t_format_v1`, `t_format_forward_compat` |
| NFR-003: Tauri surface | `graph:load` invoke + CLI | `t_cli_build`, `t_invoke_returns_arraybuffer` |
| NFR-004: Cross-platform | Rust + TS + Tauri | CI Windows + Linux |
| NFR-005: No GUI in core | `crates/ui` separado | Workspace members check |

---

## Performance Impact

| Metric | Budget | Baseline (spike) | New expected | Measured |
| --- | --- | --- | --- | --- |
| Extract 600k commits | < 30 s | 9.7 s | <= 9.7 s | gate G2 + `t_extract_within_budget` |
| Graph size | < 32 MB | 9.6 MB | 9.6 MB | `t_graph_size` |
| Load into renderer | < 2 s | 522 ms | <= 522 ms | `t_load_time` |
| Frame p50 | < 16.7 ms | 16.7 ms | <= 16.7 ms | `t_frame_p50` |
| Frame p99 | < 33.4 ms | 19.5 ms | <= 19.5 ms | `t_frame_p99` |
| Max frame | < 50 ms | 20.2 ms | <= 20.2 ms | `t_frame_max` |

Benchmark: `cargo test --package gitflowfy-core --test perf -- --nocapture` + `npm run bench` en `crates/ui`.

---

## Project Structure

```
crates/
  core/
    Cargo.toml
    src/
      lib.rs
      git/          # Feature 001
      graph/
        mod.rs
        builder.rs      # git log -> CSR -> graph.bin
        format.rs       # v1 header + CSR encode/decode
      cli/
        mod.rs
        bin/gitflowfy.rs
  ui/
    Cargo.toml        # npm-style: devDependencies para TypeScript
    package.json
    tsconfig.json
    src/
      renderer.ts       # Canvas 2D + typed arrays
      viewport.ts       # Virtualized pan/zoom
      decode.ts         # graph.bin -> typed arrays
      main.ts           # webview entry
  desktop/
    Cargo.toml
    tauri.conf.json
    src/
      main.rs
    icons/
```

**Dependencies new**:
- `crates/core`: `bytemuck` (zero-copy decode), `tempfile` (existente)
- `crates/ui`: `typescript`, `vite` (dev), `@tauri-apps/api` (invoke)
- `apps/desktop`: `tauri` v2, `tauri-plugin-shell`, `tauri-plugin-fs`

---

## Complexity Tracking

| Exception | Article | Justification | Revisit when |
| --- | --- | --- | --- |
| `bytemuck` para zero-copy decode | VIII | Zero-copy decode de `graph.bin` sin copia. No wrapper. | Si `bytemuck` MSRV/license conflict. |
| `vite` + TypeScript toolchain | VIII | Toolchain estndar para TS. No wrapper sobre renderer. | Si `vite` MSRV/license conflict. |
| `@tauri-apps/api` | VIII | Invoke IPC estndar. No wrapper. | Si Tauri API breaking change. |
| `vite` dev server en CI | VI | Requiere Node en CI. No afecta runtime. | Nunca; CI toolchain estndar. |
| `crates/ui` + `apps/desktop` nuevos crates | VII | Constitucin permite 3 crates total. | Nunca; arquitectura fijada. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| R1: Builder Rust ms lento que spike JS | Media | Budget extract > 30s | Gate G2 mide antes de commit. Si falla, optimizar (paralelizar parse, SIMD). |
| R2: Renderer TS en webview ms lento que spike Chrome | Baja | Frame budget miss | Spike ya en Chrome/Edge webview2. CI mide en Windows/Linux. |
| R3: `graph.bin` format evolution | Media | Compatibilidad rota | Version header v1. Forward compat: renderer lee v1+v2. |
| R4: Tauri webview2 no disponible en Linux CI | Media | Tests fallan | `webview2` en Windows, `webkitgtk` en Linux. CI configura ambos. |
| R5: Memoria graph + renderer > budget | Baja | OOM / frame drop | 9.6MB + renderer ~20MB << lmite. Monitor en CI. |
| R6: Incremental update complexity | Alta (si se aade luego) | Complejidad innecesaria | Deferred a feature posterior. Full rebuild 9.7s aceptable. |

---

## Plan Self-Review

- [x] Every requirement traced to decision + test
- [x] All 5 clarifications resolved in spec
- [x] Every git command spelled out literally
- [x] Headless surface defined before GUI (Phase 1-2 before Phase 3)
- [x] Fixtures specified for all in-scope Article V states (reused from 001)
- [x] Perf budget + measurement method stated (Article VI + gate G2)
- [x] Complexity exceptions justified (4, each with trigger)
- [x] Pure ASCII output
- [x] Plan stays high-level; code lives in `implementation-details/`

---

*Plan version 1.0.0 - listo para revisin constitucional*