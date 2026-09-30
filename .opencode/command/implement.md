---
name: sdd-implement
description: Implement one task from an approved task list, test first.
disable-model-invocation: true
argument-hint: "[task id, e.g. T007]"
---

Implement exactly one task.

**Task:** $ARGUMENTS

## Steps

1. If `$ARGUMENTS` is empty, show the open tasks from `tasks.md` and ask which
   one. Implement exactly one - not the phase, not the feature.

2. Read `spec.md`, `plan.md`, `tasks.md`, and `.claude/constitution.md`.

3. Delegate to the `sdd-impl` agent with the task ID and the feature path.

4. Report back: the task ID, what changed, the test that went from red to
   green **with its actual output**, the full-suite result, and any deviation
   from the plan.

## Rules

Constitution Article III: write the test, run it, watch it fail, then implement.
A test that passes before the implementation exists is a broken test.

If the task is not listed in `tasks.md`, it is not authorized. Stop and say so.
Unlisted work is how scope becomes unbounded.

If the plan turns out to be wrong, stop and report. Do not work around it and
do not silently redesign - `sdd-impl` does not own `spec.md` or `plan.md`.
