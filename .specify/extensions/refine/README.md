# spec-kit-refine

A [Spec Kit](https://github.com/github/spec-kit) extension that enables iterative specification refinement — update specs in-place, propagate changes to downstream artifacts, and track sync status across the SDD lifecycle.

## Problem

Spec Kit's `/speckit.specify` command is optimized for creating new specifications, but real-world development requires continuous iteration. When requirements change mid-implementation, teams face:

- No dedicated workflow for updating specs without creating new branches
- Plan and tasks go stale when specs change, with no warning or propagation mechanism
- No way to preview the downstream impact of a spec change before applying it
- No visibility into which artifacts are in sync and which are outdated

## Solution

The Refine extension adds four commands that close the iteration gap:

| Command | Purpose | Modifies Files? |
|---------|---------|-----------------|
| `/speckit.refine.update` | Update spec.md in-place based on new requirements or feedback | Yes — spec.md |
| `/speckit.refine.propagate` | Cascade spec changes to plan.md and tasks.md | Yes — plan.md, tasks.md |
| `/speckit.refine.diff` | Preview impact of spec changes before propagating | No — read-only |
| `/speckit.refine.status` | Show sync status between spec, plan, and tasks | No — read-only |

## Installation

```bash
specify extension add --from https://github.com/Quratulain-bilal/spec-kit-refine/archive/refs/tags/v1.0.0.zip
```

## Workflow

```
Requirements change
       │
       ▼
/speckit.refine.update    ← Update spec.md in-place
       │
       ▼
/speckit.refine.diff      ← Preview downstream impact (optional)
       │
       ▼
/speckit.refine.propagate ← Cascade changes to plan.md & tasks.md
       │
       ▼
/speckit.refine.status    ← Verify all artifacts are in sync
```

## Commands

### `/speckit.refine.update`

Updates an existing spec.md without creating a new feature branch. Handles:
- Adding new user stories (assigned next priority)
- Modifying existing requirements
- Removing requirements (marked with strikethrough, never deleted)
- Adding refinement notes with date and description
- Marking downstream artifacts (plan.md, tasks.md) as stale

### `/speckit.refine.propagate`

Surgically updates plan.md and tasks.md to reflect spec changes:
- Adds new plan sections for new user stories
- Adds new task phases with proper IDs, dependencies, and story labels
- Marks removed items with strikethrough (never deletes)
- Updates the Execution Wave DAG if present
- Validates consistency after propagation
- Removes staleness warnings after successful update

### `/speckit.refine.diff`

Read-only impact analysis before propagating:
- Lists all refinement changes detected
- Shows which plan sections are affected
- Shows which tasks are affected (by ID)
- Runs consistency checks (story coverage, requirement traceability)
- Recommends next action

### `/speckit.refine.status`

Dashboard showing artifact synchronization:
- Shows status of all artifacts (spec, plan, tasks, research, data-model, contracts)
- Identifies stale artifacts that need propagation
- Shows refinement history
- Counts user stories, requirements, and tasks

## Hooks

The extension registers optional hooks:
- **after_specify**: Shows sync status after generating a new spec
- **after_plan**: Shows sync status after generating a plan

## Design Decisions

- **Never delete content** — removed items are marked with strikethrough and a reason, preserving change history
- **Never create new branches** — refine operates on the current branch, keeping the workflow lightweight
- **Surgical updates** — propagate modifies only affected sections, not full regeneration
- **Staleness tracking** — stale artifacts get visible warnings so no one implements from outdated docs
- **Read-only commands** — diff and status are safe to run anytime without side effects
- **Traceability** — every change is tracked with dates and descriptions

## Requirements

- Spec Kit >= 0.4.0

## Related

- Issue [#1191](https://github.com/github/spec-kit/issues/1191) — Spec-Driven Editing Flow: Can't Easily Update or Refine Existing Specs (101+ upvotes)

## License

MIT
