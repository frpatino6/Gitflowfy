---
name: sdd-plan
description: Turn an approved spec into a technical implementation plan.
disable-model-invocation: true
argument-hint: "[feature name or path]"
---

Create an implementation plan for a feature.

**Feature:** $ARGUMENTS

## Steps

1. Locate the spec. If `$ARGUMENTS` is empty, list `specs/` and ask which one.
   If no spec exists, stop - the plan comes after the spec.

2. Read the spec, `.claude/constitution.md`, and `AGENTS.md`.

3. Delegate to the `sdd-plan` agent. It produces `plan.md` plus any of
   `research.md`, `data-model.md`, `contracts/`, and
   `implementation-details/` that the feature needs.

4. Report back: the paths written, the blockers that need the user's answer,
   and any constitution gate that passed only with a written justification.

## Rules

Constitution Article I - every mutating operation is spelled out as a literal
`git` command, not a description of one.

Constitution Article IV - the headless surface is designed before any UI. If
the feature cannot work without a window, that is a finding to report, not a
constraint to design around.

Run the Phase -1 gates honestly. A gate that fails and gets justified anyway
needs a specific reason and a "revisit when" trigger. A gate that is quietly
ignored makes all nine articles decorative.
