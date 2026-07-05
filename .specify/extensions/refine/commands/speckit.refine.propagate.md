---
description: "Propagate spec changes downstream to plan.md and tasks.md"
---

# Propagate Spec Changes

After refining a specification with `/speckit.refine.update`, use this command to cascade changes downstream to plan.md and tasks.md without regenerating them from scratch.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The user may specify which artifacts to propagate to (e.g., "only plan" or "plan and tasks").

## Prerequisites

1. Verify a spec-kit project exists by checking for `.specify/` directory
2. Locate the current feature's spec directory (same logic as `/speckit.refine.update`)
3. Verify spec.md exists and has been refined (check for `**Refined**:` entries or `**Status**: Refined`)
4. Check which downstream artifacts exist: plan.md, tasks.md

## Outline

1. **Load artifacts**: Read from the current feature directory:
   - **Required**: `spec.md` (the refined specification)
   - **Target**: `plan.md` and/or `tasks.md` (artifacts to update)
   - **Optional**: `research.md`, `data-model.md`, `contracts/` (for context)

2. **Identify what changed**: Parse the refinement notes in spec.md:
   - Look for `**Refined**:` entries to understand the change history
   - Look for `~~removed~~` items to identify dropped requirements
   - Compare user stories, requirements, and success criteria against plan.md/tasks.md

3. **Propagate to plan.md** (if it exists and user didn't exclude it):
   - **Added user stories**: Add corresponding sections to the plan (project structure, complexity tracking)
   - **Modified requirements**: Update the Technical Context, Project Structure, or Complexity Tracking sections as needed
   - **Removed requirements**: Mark corresponding plan sections with strikethrough and reason
   - **Preserve**: All existing plan structure, formatting, and sections not affected by the change
   - Remove any `⚠️ **STALE**` warning from the top of the file
   - Add a propagation note:
     ```
     **Propagated**: [DATE] — Updated from spec.md refinement
     ```

4. **Propagate to tasks.md** (if it exists and user didn't exclude it):
   - **Added user stories**: Add a new Phase section with tasks following the existing format (`[ID] [P?] [Story?] Description [(depends on ...)]`)
   - **Modified requirements**: Update affected tasks in-place — adjust descriptions, dependencies, or file paths
   - **Removed requirements**: Mark affected tasks with `~~[REMOVED]~~` prefix and strikethrough, do not delete
   - **Renumber if needed**: If new tasks are added, assign IDs continuing from the last used ID
   - **Update Execution Wave DAG**: If the tasks.md has a Wave DAG section, regenerate it to reflect the new task dependencies
   - **Update Dependencies section**: Adjust phase dependencies and user story dependencies
   - Remove any `⚠️ **STALE**` warning from the top of the file
   - Add a propagation note:
     ```
     **Propagated**: [DATE] — Updated from spec.md refinement
     ```

5. **Validate consistency**: After propagation, verify:
   - Every user story in spec.md has a corresponding phase in tasks.md
   - Every requirement in spec.md is traceable to at least one task
   - No orphaned tasks reference removed requirements
   - Dependencies still form a valid DAG (no circular dependencies)

6. **Report**: Output a summary:
   - What changed in each artifact
   - How many tasks were added, modified, or marked removed
   - Any consistency warnings found during validation
   - Suggest next step: `/speckit.refine.diff` to review changes, or `/speckit.implement` to continue building

## Rules

- **Never regenerate from scratch** — propagate surgically updates only affected sections
- **Never delete content** — mark removed items with strikethrough
- **Preserve all formatting** — match existing artifact styles
- **Track changes** — always add propagation notes with dates
- **Remove staleness warnings** — after successful propagation, clear the stale markers
- **Maintain traceability** — every spec requirement must map to plan/task items
