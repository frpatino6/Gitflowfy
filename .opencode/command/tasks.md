---
name: sdd-tasks
description: Break an approved plan into an ordered, test-first task list.
disable-model-invocation: true
argument-hint: "[feature name or path]"
---

Break a plan into an executable task list.

**Feature:** $ARGUMENTS

## Steps

1. Locate `plan.md` for the feature. If it does not exist, stop - planning and
   decomposition are separate decisions, and merging them hides the reasoning.

2. Read `plan.md`, `spec.md`, and `.claude/constitution.md`.

3. Delegate to the `sdd-tasks` agent to write `tasks.md` from
   `.claude/templates/tasks-template.md`.

4. Report back: the task count, the wave structure, which tasks are
   parallelizable, and anything in the plan that decomposed badly.

## Rules

Constitution Article III is structural, not advisory. Every test task precedes
the implementation task it validates. If the plan listed them the other way
around, the task list corrects it - the constitution outranks the plan.

Phase 0 is the safety net: fixture repositories and the differential harness.
Nothing else starts until it passes. If the plan omitted it, add it and say so.

Every task traces to an FR or NFR. A task traceable to no requirement is
speculative - cut it, or say why it is there.
