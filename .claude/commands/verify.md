---
name: sdd-verify
description: Run the gates and collect evidence for whether a feature does what its spec says.
disable-model-invocation: true
argument-hint: "[feature name or path]"
---

Verify a feature against its spec.

**Feature:** $ARGUMENTS

## Steps

1. Locate the feature's spec, plan, and task list. If `$ARGUMENTS` is empty,
   list `specs/` and ask which.

2. Delegate to the `sdd-verify` agent.

3. Report the evidence table and the verdict verbatim.

## Rules

The verdict is `verified` or `not verified`. There is no third option.

An unverified acceptance scenario is not a pass. If a scenario cannot be
exercised, it is reported as unverified - not quietly dropped.

If the performance benchmark could not run, that is `unverified`, not `pass`.
A single-platform result is not a cross-platform result; say which platform you
ran on.

Never soften a failure into a caveat. The value of this step is entirely
downstream of it being trustworthy.
