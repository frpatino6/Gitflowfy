# Research: the stack decision

The evidence behind the stack choice in `plan.md`. Recorded so the decision can be
re-examined rather than re-argued, and so a later feature that outgrows the choice can
see what it was traded against.

## Question

Rust core plus a TypeScript renderer in Tauri v2, or one language in Node?

## What the spike already proved

`spike/` is JavaScript and it is validated against 599,555 commits from
`llvm/llvm-project` (partial clone, Intel HD 4600):

| Metric | Result |
| --- | --- |
| Extract and build | 9.7 s |
| Graph binary | 9.6 MB |
| Renderer load | 522 ms |
| Frame p50 | 16.7 ms |
| Frame p99 | 19.5 ms |
| Worst frame | 20.2 ms |

Every Constitution Article VI budget is met with margin. Discarding this is not free,
so the decision had to earn the discard.

## What the spike is made of

Reading `spike/build-graph.mjs` rather than assuming its shape mattered. The build is:

1. `spawn git log --all --topo-order --format=...` with a `0x1f` separator
2. Stream stdout, split lines, fill `oidByRow`, `subjectByRow`, `parentOidFlat`
3. Resolve parent OIDs to row indices into a `Uint32Array`
4. Assign lanes into an `Int32Array` with a free-lane pool
5. Pack five typed arrays into one `ArrayBuffer` and write `graph.bin`

The renderer reads that blob directly. The design is a CSR layout in typed arrays, and
the binary is the contract between the two halves.

Two observations follow:

- **The renderer half is web technology and stays web technology.** Canvas with typed
  arrays is what produced 19.5 ms p99. Tauri hosts a webview, so this half of the
  spike survives without modification.
- **The builder half is process-and-parse work**, which is the part the core
  eventually owns in Rust. Porting it is mechanical: same algorithm, same five arrays,
  same binary format. Whether the Rust port is faster or slower than the measured
  9.7 s is **not claimed**; it is re-measured at feature 002 under Article VI, and a
  number is not asserted until one exists.

So the discard is smaller than it looks: a ~160-line script belonging to feature 002,
not a validated renderer.

## The binding constraint

SC-010 sets a 5 ms median per-invocation overhead budget over raw `git`. Our overhead
is dominated by process spawn, because Git's own runtime is the reference point and is
excluded.

- Rust `std::process::Command` is expected to have sub-millisecond spawn cost on
  Windows with no runtime tax.
- Node `child_process.spawn` is expected to carry higher per-call overhead on Windows,
  and the differential harness spawns hundreds of git processes per run, which is the
  pattern that would amplify any difference.

**Both are expectations, not measurements.** Nothing in this repository benchmarks
them. The earlier version of this file stated them as facts with "measurably" and
"sub-millisecond" attached, which is exactly the kind of unmeasured performance claim
Article VI forbids. Gate G2 measures the *chosen* stack's real overhead before the
operation surface is written, and this section records the expectation it is testing
rather than a result it has already got.

The budget is probably reachable in Node. "Probably" is the problem: this is a
commitment the spec already made, and a stack that meets a published number with thin
margin is a stack that fails that number on a slower machine.

Gate G2 measures this before any code is written, on the actual target machine. That is
the point of making it a gate: the decision is falsifiable rather than asserted.

## Why not Electron

- Ships a Chromium per instance. That memory competes directly with the 9.6 MB graph
  and the render loop whose budgets we are defending.
- Provides no benefit here that Tauri does not, and the process-spawn cost is the same
  Node cost the budget is worried about.
- The product thesis is that we do not own a parallel model of the repository. A
  150 MB runtime is a poor expression of "we are a thin honest layer over your git".

## Why not all-Rust

Tempting, since the core is Rust. Rejected because the validated renderer work is
Canvas plus typed arrays. A Rust renderer means either a webview bridge or a native
GPU path, and both throw away the measured 19.5 ms p99 for a re-implementation nobody
has validated. Keeping the webview is free; the cost is an IPC boundary, which
Article IV requires anyway.

## The honest weakness in this decision

**The graph builder is paid for twice**: once as the JavaScript spike, once as the
Rust port. This is real duplicated work, and it is the price of the spawn budget. If
gate G2 later shows Node meets 5 ms comfortably on target hardware, the correct move
is to reconsider -- a single-language Node core would delete the port, delete the IPC
boundary for the graph, and reuse the spike directly.

That is why the gate exists and why it measures before the code is written. This
decision is conditional on a number, and the number has not been taken yet.

## Interop boundary

Tauri's command surface. The graph crosses it as raw bytes (`graph.bin`), not JSON,
which is precisely why the CSR layout is a packed binary rather than an object graph --
a decision the spike already made, and one that would be wrong if the renderer were
not a webview.

Feature 001 creates neither the UI nor the desktop crate. The renderer decision binds
from feature 002.
