---
description: Fill this template when specifying a feature. Focus on WHAT and WHY. No implementation details.
---

# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[NNN-feature-name]`
**Status**: Draft | Under Review | Approved
**Created**: YYYY-MM-DD
**Constitution**: `.claude/constitution.md` v1.0.0

## Purpose

<!-- One paragraph. What problem does this solve for the user, and why does it
     matter relative to GitKraken/Fork/Extensions? If you cannot name a concrete
     weakness in an incumbent that this feature beats, the feature is not
     justified. -->

## Why Now

<!-- What forces this work at this point in the project? -->

## Scope

### In Scope
- 

### Out of Scope
- <!-- Explicitly state what this does NOT do, to stop scope creep later. -->

---

## User Stories

> Each story is independently valuable. Priority: P1 (must ship), P2 (should),
> P3 (nice to have). Maximum 5 stories per feature.

### Story 1 - [title] (P1)

**As a** [role],
**I want** [capability],
**so that** [benefit].

**Independent test**: [How is this story verified without other stories?]

**Acceptance scenarios**:
1. **Given** [state], **when** [action], **then** [observable outcome]
2. **Given** [edge state], **when** [action], **then** [outcome]

---

## Requirements

### Functional

- **FR-001**: The system SHALL ...
- **FR-002**: ...

### Non-Functional

<!-- Pull thresholds from Constitution Article VI where they apply. Do not
     invent numbers; cite the article. -->

- **NFR-001**: [performance / memory / responsiveness requirement]

### Constitutional Constraints

<!-- Which articles bind this feature, and what do they forbid here?
     This is not boilerplate - it prevents the planner from proposing
     something that Article I or IV rules out. -->

- Article [N]: [how it constrains this feature]

### Edge Cases

<!-- Constitution Article V requires unusual repo states to be handled.
     List the states this feature touches. -->

| Case | Expected behavior |
| --- | --- |
| | |

---

## Success Criteria

<!-- Must be measurable. "Works well" is not a criterion. -->

- **SC-001**: [metric with a number]
- **SC-002**: [metric with a number]

---

## Competitive Differentiation

<!-- Be honest. If a competitor already does this, say how we beat them.
     If we are merely matching them, flag it: that is not a differentiator. -->

| Capability | GitKraken | Fork | Extensions | Our edge |
| --- | --- | --- | --- | --- |
| | | | | |

---

## Open Questions

<!-- Anything the user must decide. Keep these as explicit questions; do not
     guess. -->

| # | Question | Blocks | Owner |
| --- | --- | --- | --- |
| Q1 | | | |

---

## Clarification Check

> Spec Kit's failure mode is the model quietly inventing details. Every
> ambiguity MUST be marked, not guessed.

- [ ] No implementation details (no tech stack, no file paths, no APIs)
- [ ] Every requirement is testable and unambiguous
- [ ] Every acceptance scenario has observable outcomes
- [ ] All ambiguities marked `[NEEDS CLARIFICATION]` or listed in Open Questions
- [ ] Success criteria are numeric
- [ ] Out of scope is explicit
- [ ] Constitutional constraints identified
- [ ] Competitive claims are honest about parity vs. advantage
