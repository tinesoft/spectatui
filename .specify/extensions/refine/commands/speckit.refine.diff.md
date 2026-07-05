---
description: "Show what changed in spec.md and predict downstream impact on plan and tasks"
---

# Diff Spec Changes

Preview the impact of spec.md changes on downstream artifacts before propagating. Use this to understand the blast radius of a refinement.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Prerequisites

1. Verify a spec-kit project exists by checking for `.specify/` directory
2. Locate the current feature's spec directory
3. Verify spec.md exists

## Outline

1. **Load all artifacts**: Read from the current feature directory:
   - `spec.md` (check for `**Refined**:` entries)
   - `plan.md` (if exists)
   - `tasks.md` (if exists)

2. **Extract refinement history**: Parse all `**Refined**:` entries in spec.md to build a change log:
   ```
   ## Refinement History
   - [DATE]: [Description of change]
   - [DATE]: [Description of change]
   ```

3. **Analyze spec changes**: For each refinement, classify the impact:

   | Change Type | Spec Impact | Plan Impact | Tasks Impact |
   |-------------|-------------|-------------|--------------|
   | New user story | New section added | New structure/complexity entry | New phase with tasks |
   | Modified requirement | Updated requirement text | May affect technical context | May affect task descriptions |
   | Removed requirement | Strikethrough marking | Sections to mark removed | Tasks to mark removed |
   | Changed priority | Reordered user story | May affect implementation order | May reorder phases |
   | New success criterion | Added to criteria list | Minimal | May need verification task |

4. **Generate impact report**: Output a structured diff report:

   ```markdown
   # Spec Refinement Impact Report

   ## Changes Detected
   - [List each change from refinement notes]

   ## Downstream Impact

   ### plan.md
   - **Status**: [Up to date | Stale | Does not exist]
   - **Sections affected**: [List of plan sections that need updates]
   - **Estimated changes**: [Number of sections to add/modify/remove]

   ### tasks.md
   - **Status**: [Up to date | Stale | Does not exist]
   - **Tasks affected**: [List of task IDs that need updates]
   - **New tasks needed**: [Count]
   - **Tasks to mark removed**: [Count]
   - **Dependency changes**: [Yes/No — describe if yes]

   ## Consistency Check
   - [ ] All spec user stories have plan sections: [Pass/Fail]
   - [ ] All spec requirements have task coverage: [Pass/Fail]
   - [ ] No orphaned tasks from removed requirements: [Pass/Fail]

   ## Recommended Action
   - Run `/speckit.refine.propagate` to apply these changes
   - Or run `/speckit.refine.propagate only plan` to update plan.md first
   ```

5. **Report**: Output the impact report. Do not modify any files — this command is read-only.

## Rules

- **Read-only** — this command never modifies any files
- **Always show impact** — even if no downstream artifacts exist, report what would be needed
- **Be specific** — list exact section names and task IDs affected, not vague summaries
