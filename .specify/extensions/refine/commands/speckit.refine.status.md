---
description: "Show sync status between spec, plan, and tasks — identify stale artifacts"
---

# Artifact Sync Status

Show the synchronization status between spec.md, plan.md, and tasks.md. Identifies which artifacts are stale and need propagation.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Prerequisites

1. Verify a spec-kit project exists by checking for `.specify/` directory
2. Locate the current feature's spec directory

## Outline

1. **Locate artifacts**: Find all artifacts in the current feature directory:
   - `spec.md` — the source of truth
   - `plan.md` — implementation plan
   - `tasks.md` — task breakdown
   - `research.md` — research notes
   - `data-model.md` — data model
   - `contracts/` — interface contracts

2. **Check each artifact**: For each file that exists, determine:
   - **Last modified**: File modification timestamp
   - **Has refinement notes**: Check for `**Refined**:` entries (spec.md)
   - **Has propagation notes**: Check for `**Propagated**:` entries (plan.md, tasks.md)
   - **Has staleness warning**: Check for `⚠️ **STALE**` marker
   - **Spec user story count**: Number of user stories in spec.md
   - **Plan section count**: Number of `##`-level sections in plan.md
   - **Task count**: Number of `- [ ]` items in tasks.md

3. **Determine sync status**: Compare artifacts to assess synchronization:

   | Condition | Status |
   |-----------|--------|
   | spec.md refined after plan.md last propagated | plan.md is **STALE** |
   | spec.md refined after tasks.md last propagated | tasks.md is **STALE** |
   | plan.md has `⚠️ **STALE**` marker | plan.md is **STALE** |
   | tasks.md has `⚠️ **STALE**` marker | tasks.md is **STALE** |
   | No refinement notes and no staleness markers | All artifacts are **IN SYNC** |
   | Artifact does not exist | **MISSING** |

4. **Output status dashboard**:

   ```markdown
   # Artifact Sync Status: [Feature Name]

   | Artifact | Status | Last Modified | Notes |
   |----------|--------|---------------|-------|
   | spec.md | ✅ Current | [date] | [N user stories, M requirements] |
   | plan.md | ⚠️ Stale | [date] | Refined on [date], not yet propagated |
   | tasks.md | ⚠️ Stale | [date] | [X total tasks, Y completed] |
   | research.md | ✅ Present | [date] | — |
   | data-model.md | ❌ Missing | — | — |
   | contracts/ | ❌ Missing | — | — |

   ## Refinement History
   - [DATE]: [Change description]

   ## Recommended Actions
   1. Run `/speckit.refine.propagate` to update stale artifacts
   2. Run `/speckit.refine.diff` to preview impact before propagating
   ```

5. **Report**: Output the dashboard. Do not modify any files — this command is read-only.

## Rules

- **Read-only** — this command never modifies any files
- **Always show all artifacts** — include missing ones so user knows what to create
- **Use clear status indicators** — ✅ for current, ⚠️ for stale, ❌ for missing
