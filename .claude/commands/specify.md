---
name: sdd-specify
description: Start a new feature by writing its specification. Use when a user describes a feature that has no spec yet.
disable-model-invocation: true
---

Create a specification for a new feature.

**User's request:** $ARGUMENTS

## Steps

1. Read `.claude/constitution.md` and `AGENTS.md` for context.

2. Scan `specs/` to find the highest existing feature number. The new one is
   that plus 1, zero-padded to three digits.

3. Slugify the feature name: lowercase, hyphenated, no leading articles.
   Create `specs/NNN-feature-name/`.

4. Delegate to the `sdd-spec` agent to write `spec.md` from
   `.claude/templates/spec-template.md`. Give it the user's request verbatim
   plus any context from this conversation.

5. Report back: the path, the feature number and name, how many
   `[NEEDS CLARIFICATION]` markers are open, and what scope you cut.

## Rules

The spec states WHAT and WHY, never HOW. No tech stack, no file paths, no
function names. If the plan phase needs those, that is what the plan is for.

Every ambiguity gets marked, not guessed. Do not stop to ask the user
questions - produce the complete document with the markers in place, then list
them at the end so they can be answered in one pass.

## Do not continue past this step

Do not write the plan, do not write code. The spec is reviewed and approved
first, or the whole methodology collapses into writing code and documenting it
afterwards.
