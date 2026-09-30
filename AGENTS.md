# SDD Harness Verification

Let me verify the harness is wired correctly before we start specifying.

**Tooling context**

- Working directory: `D:\Projects\Gitflowfy`
- Platform: win32, PowerShell
- This is a **Git client project** (Gitflowfy) - a competitor to GitKraken/Git
  Fork/Git Extensions. Binding rules live in `.claude/constitution.md`.

**What exists now**

- `.claude/constitution.md` - nine binding articles (I shell out to git, II
  reflog-only state, III test-first, IV CLI-first, V odd repo states, VI perf
  budgets, VII <=3 crates, VIII anti-abstraction, IX integration-first)
- `.claude/agents/` - sdd-spec, sdd-plan, sdd-tasks, sdd-impl, sdd-review,
  sdd-verify
- `.claude/templates/` - spec, plan, tasks templates
- `.claude/skills/` - perf-benchmark, repo-states
- `.claude/commands/` - specify, plan, tasks, implement, verify
- `.opencode/` - hardlinks to the above (same file identity, no copies)
- `spike/` - validated perf harness; 599,555 commits, p99 19.5 ms
- `specs/` - empty except README

**Verified facts (do not re-derive)**

- Node 20.20.2. The CDP client needs `--experimental-websocket` on Node 20.
- The spike harness owns its own server and Chrome; nothing outlives a run.
- `spike/cache/` holds ~134 MB of generated data. Regenerable, gitignored.

**Working agreements**

- The spec is the tiebreaker. If code and spec disagree, fix the spec first.
- No implementation without a spec authorizing it.
- Ambiguity gets marked `[NEEDS CLARIFICATION]`, never guessed.
- No performance claim without a measurement behind it.
- The user wants SDD used for real, not as documentation theater.

