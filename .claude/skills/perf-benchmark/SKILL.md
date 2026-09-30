---
name: perf-benchmark
description: How to measure Gitflowfy performance against the Article VI budgets. Use when the graph model or renderer changes, or when a performance claim needs a real number behind it.
---

# Performance benchmarking

Constitution Article VI makes performance a contract. This skill is how you
produce the number.

## The harness

`spike/run-spike.mjs` runs headless Chrome over the DevTools Protocol. It
serves the graph itself, so no leftover process survives the run.

```bash
node --experimental-websocket spike/run-spike.mjs
```

`--experimental-websocket` is required on Node 20 for the DevTools Protocol
client. On Node 22+, plain `node` works.

The harness starts and stops its own server and Chrome. If a run is killed
mid-flight, check port 5178 and the `chrome-spike` user-data directory.

## Budgets

From `.claude/constitution.md` Article VI. These are pass/fail, not goals.

| Metric | Budget | Spike baseline |
| --- | --- | --- |
| Extract 600k commits | < 30 s | 9.7 s |
| In-memory graph size | < 32 MB | 9.6 MB |
| Load into renderer | < 2 s | 522 ms |
| Frame p50 | < 16.7 ms | 16.7 ms |
| Frame p99 | < 33.4 ms | 19.5 ms |
| Worst frame | < 50 ms | 20.2 ms |

## Reading the numbers

- **p50** is the typical frame. If p50 fails, something regressed broadly.
- **p99** is what a user perceives as stutter. This is the number that matters.
- **max** catches a single pathological frame - often a GC pause or a
  miscalculated culling bound.

A regression of more than 10% from baseline fails the gate even if still under
budget. The budgets are the floor, not the target.

## The stress test is the real test

Idle FPS is meaningless - nothing is being drawn. The harness's stress mode
traverses the entire graph, animating `offsetY` end to end at `rowH = 4`, so
the viewport holds the maximum number of rows. That is the worst case a user
can scroll into.

Any rendering change must be measured under that load, not at rest.

## Measurement discipline

- Never report a number you did not measure.
- Never let a benchmark pass be inherited from a previous run.
- If the benchmark could not run, the result is `unverified`, not `pass`.
- Record the git version, the repo fixture, and the resolution alongside the
  numbers. A frame time without that context cannot be compared later.

## Choosing a fixture

The 600k-commit fixture is mandatory for Article VI. `llvm/llvm-project` is the
reference: 599,555 commits, cloned with `--filter=blob:none`.

Its merge density is the weakness - only 5 merges and 5 lanes, so the lane
algorithm is barely exercised. For graph-algorithm work, a denser fixture is
also needed. Two that stress it harder:

- `torvalds/linux` - 1.5M commits, ~6.4 GB
- Repositories with long-lived release branches and heavy back-merging

If you add a denser fixture, add it to the harness and record its numbers. The
gap between the current fixture and a real monorepo is where the incumbents
break, and it is where our claim lives or dies.
