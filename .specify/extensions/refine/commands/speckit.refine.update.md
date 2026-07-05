---
description: "Update an existing spec.md in-place based on new requirements or feedback"
---

# Refine Specification

Update an existing specification in-place without creating a new feature branch. Use this when requirements change, feedback arrives, or you need to iterate on an existing spec.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The user input describes what to change in the existing specification.

## Prerequisites

1. Verify a spec-kit project exists by checking for `.specify/` directory
2. Identify the current feature by checking for an active feature branch (pattern: `###-feature-name` or `YYYYMMDD-HHMMSS-feature-name`)
3. If no feature branch is detected, look for the most recently modified `specs/*/spec.md`
4. If no spec.md is found, inform the user and suggest running `/speckit.specify` first

## Outline

1. **Locate the spec**: Find the current feature's spec.md:
   - Check git branch name for feature number/prefix
   - Map to `specs/{branch-name}/spec.md`
   - If not on a feature branch, prompt the user to select from available specs in `specs/*/spec.md`

2. **Read current artifacts**: Load the following files from the feature directory:
   - **Required**: `spec.md` (the specification to refine)
   - **Optional**: `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `contracts/`
   - Note which optional artifacts exist — they will need propagation later

3. **Understand the change request**: Analyze the user's input to determine:
   - Which sections of spec.md are affected
   - Whether this is an additive change (new requirements), a modification (changed requirements), or a removal (dropped requirements)
   - Whether the change affects user stories, requirements, success criteria, or assumptions

4. **Apply the update**: Modify spec.md in-place:
   - Preserve all existing sections and formatting
   - Update only the sections affected by the change request
   - If adding a new user story, assign the next priority number (P4, P5, etc.)
   - If modifying requirements, update both the requirement and any affected user stories
   - If removing requirements, mark them as `~~removed~~` with a brief reason rather than deleting
   - Update the `**Status**` field to `Refined` and add a refinement note:
     ```
     **Refined**: [DATE] — [Brief description of what changed]
     ```

5. **Mark downstream artifacts as stale**: After updating spec.md, check which downstream artifacts exist and append a staleness warning to each:
   - If `plan.md` exists, prepend:
     ```
     > ⚠️ **STALE**: spec.md was refined on [DATE]. Run `/speckit.refine.propagate` to update this plan.
     ```
   - If `tasks.md` exists, prepend the same warning
   - This ensures no one accidentally implements from outdated artifacts

6. **Report**: Output a summary:
   - What sections were updated in spec.md
   - Which downstream artifacts are now stale
   - Suggest next step: `/speckit.refine.propagate` to cascade changes, or `/speckit.refine.diff` to preview impact first

## Rules

- **Never create a new feature branch** — refine works on the current branch
- **Never delete content** — mark removed items with strikethrough and a reason
- **Preserve formatting** — match the existing spec.md style exactly
- **Preserve section order** — do not reorder sections
- **Track changes** — always add a refinement note with date and description
- **Mark staleness** — always warn downstream artifacts about spec changes
