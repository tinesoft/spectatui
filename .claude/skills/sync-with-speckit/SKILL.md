---
name: sync-with-speckit
description: Detects whether Spectatui has fallen behind upstream GitHub Spec-Kit by capturing the real `specify` CLI surface (via uvx, not by reading prose), reading spec-kit's actual upstream source — slash-command templates, document skeletons, shared scripts — for features invisible to `--help`, and comparing all of that against Spectatui's actual code. Use this whenever the user asks whether Spectatui supports something from Spec-Kit, mentions Spec-Kit/specify-cli releases or changelogs, asks what's new upstream, wants a gap analysis or integration roadmap between Spectatui and Spec-Kit, or wants to know if the CLI wrapper is out of date — even if they just say "check for spec-kit updates" or "are we behind on spec-kit" without naming this skill.
compatibility: Requires `uvx` on PATH and network access to github.com / api.github.com / raw.githubusercontent.com
metadata:
    author: spectatui
user-invocable: true
disable-model-invocation: false
---

# Sync With Spec-Kit

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). In particular: if the user explicitly asks to force a full re-analysis, that request overrides the version gate in step 2 — proceed straight to step 3 regardless of what the version comparison finds. The gate exists to avoid redundant work when nothing's changed, not to override an explicit user request to redo the work anyway.

## Goal

Detect when upstream Spec-Kit (`github/spec-kit`) has shipped a new `feat`/breaking release since Spectatui last analyzed it, discover the _actual_ current `specify` CLI surface by running it (not by reading prose), compare that against Spectatui's real implementation (code, not just docs), and produce an actionable integration-gap report — without redoing the expensive analysis for patch-only upstream releases.

## Operating Constraints

- **Read-only against upstream**: two sanctioned mechanisms only. (1) `specify ... --help` invocations — never a mutating `specify` subcommand (e.g. `init`, `check`, `self upgrade`, `extension add`). (2) Plain `WebFetch` GETs of raw upstream files (templates, scripts, README, releases, repo tree listings) — this is not a CLI call and doesn't touch a working copy, so it's fine even though it goes beyond `--help`.
- **Read-only against Spectatui source**: never edit anything under `crates/`, `specs/`, `design/core/`, `design/ui/`, `README.md`, or `.specify/`.
- **Only writes under `design/memory/`**: `tui-feature-analysis.md`, `speckit-cli-surface.md`, `sync-log.md`.
- Treat the "Out of Scope" section of `specs/001-spectatui-dashboard-mvp/spec.md` (catalog search/browse of not-yet-installed items; CLI self-check/self-upgrade) as a standing baseline to re-check every full analysis — these are known gaps, watch for upstream CLI surface that would close them.

## Execution Steps

### 1. Read prior sync state

Read `design/memory/sync-log.md`. If it doesn't exist, create it with empty frontmatter and a "No prior syncs." body:

```yaml
---
last_synced_at: null
speckit_local_version: null
speckit_remote_version: null
last_speckit_released_version: null
total_syncs: 0
---
```

### 2. Cheap version gate (always runs first)

WebFetch `https://api.github.com/repos/github/spec-kit/releases` (or `.../releases/latest`) to get the latest release tag, e.g. `v0.13.0`.

- Unconditionally set `last_speckit_released_version` in the frontmatter to this tag — this field always reflects upstream reality, every run, regardless of what happens next.
- Parse this tag and the cached `speckit_remote_version` as semver (`major.minor.patch`).
- **If `major` and `minor` are unchanged** versus `speckit_remote_version` (only `patch` moved, or nothing moved — no new `feat`/breaking release under Spec-Kit's conventional-commit versioning) **and the user has not explicitly asked to force a re-analysis**:
    - Prepend to `sync-log.md`'s body: `## <today> — sync #<total_syncs+1> (checked, still on <major>.<minor>.x — no feat/major release since <speckit_remote_version>)`
    - Update `last_synced_at`, increment `total_syncs`. Leave `speckit_local_version`/`speckit_remote_version` untouched.
    - Print a one-line summary to the user and **stop** — do not proceed to step 3.
- **If `major` or `minor` increased**, `speckit_remote_version` is `null` (first run ever), **or the user explicitly asked to force a re-analysis**: proceed to step 3. In the forced-but-version-unchanged case, say so plainly in step 3f's log entry (see the note there) rather than writing a version arrow that looks like a copy-paste no-op.

### 3. Full analysis (only runs when the gate passes)

#### 3a. TUI feature analysis — from ground truth, not just docs

Read, in order:

- `README.md` (✨ Features section) and `specs/001-spectatui-dashboard-mvp/spec.md` (Functional Requirements + Out of Scope section) — the _claimed_ feature set.
- `crates/spectatui-core/src/speckit/{mod.rs,cli.rs,registry.rs,workflow.rs,watch.rs}` — the actual Spec-Kit integration: which `specify` subcommands/concepts (extensions/presets/integrations/workflows/catalogs) are really wired up, and how (`SpecifyCliClient` command construction).
- `crates/spectatui/src/{app.rs,ui/*,main.rs,event.rs}` — which screens/panels actually exist in the TUI, plus how spec-kit gets wired in at startup (`main.rs`'s `specify_cli_available()` probe, `watch::start_watcher`) and re-exported into the event loop (`event.rs`).
- `design/ui/Spectatui.dc.html` — what's designed but not necessarily built yet.
- `design/core/spectatui-archi-design.md` — architecture intent.

Re-derive every claim from this fresh read — don't carry forward file:line citations or "Missing" classifications from a previous `tui-feature-analysis.md`/`sync-log.md` without independently re-checking them. Cached reports go stale (code moves between files, or a capability turns out to be implemented through a path you didn't think to check yet), and citing a prior report's claim as if you verified it yourself is how errors compound across syncs. Before calling anything "Missing," grep for the capability by name/keyword across the whole `crates/` tree, not just the one file a prior report pointed at — a feature can exist via a different mechanism than the one you expected (e.g. a direct API/HTTP call standing in for a CLI subcommand nobody wired up).

Reconcile mismatches explicitly (e.g. "README claims X but no code implements it" or "code does X but README/spec don't mention it"). Write `design/memory/tui-feature-analysis.md`:

```yaml
---
generated_at: '<ISO-8601>'
speckit_version: '<latest tag from step 2>'
---
```

followed by a concise inventory of supported Spec-Kit concepts and any doc/code mismatches found.

#### 3b. Capture the live upstream CLI surface

Run the bundled script — it walks `specify --help` and recurses into every subcommand's `--help` by parsing each block's own "Commands" box, so it always reflects the real tree even as upstream adds/removes commands (a hardcoded command list would silently drift stale the moment that happens):

```bash
bash .claude/skills/sync-with-speckit/scripts/capture-cli-surface.sh <latest-tag>
```

It only ever invokes `--help` (never a mutating `specify` subcommand) and needs network access for `uv` to fetch the pinned git ref. Its stdout is the full command-tree document — capped at depth 6 with an explicit `SKIPPED` marker if anything ever nests deeper (so a limit shows up as a visible note, not a silent gap). Expect this to take several minutes (`uv` resolves and builds a fresh venv for the pinned ref, then the script makes ~20-30 sequential `--help` calls) — it can exceed a 2-minute foreground timeout, so run it in the background rather than treating a timeout as a failure.

#### 3c. Read upstream source, not just `--help`

`specify --help` only exposes the packaging/installer CLI surface (`init/check/self/extension/preset/integration/workflow` catalog management). It does **not** surface spec-kit's actual AI-agent-facing feature set: the slash-command templates that get installed into a project and drive Spec-Driven Development (`/speckit.specify`, `/speckit.plan`, `/speckit.tasks`, `/speckit.clarify`, `/speckit.analyze`, `/speckit.constitution`, `/speckit.checklist`, `/speckit.implement`, `/speckit.converge`, `/speckit.taskstoissues`), nor the document skeletons those commands fill in, nor the shared automation scripts behind them. These are real candidates worth comparing against Spectatui even though no `--help` output ever mentions them — so read them directly:

- List the repo tree at the pinned tag via `https://api.github.com/repos/github/spec-kit/git/trees/<latest-tag>?recursive=1` first. Treat the file list below as a known baseline to check for, not exhaustive — tags can add, rename, or remove templates, and the tree listing is how you catch that.
- WebFetch (pinned to the same tag, e.g. `https://raw.githubusercontent.com/github/spec-kit/<latest-tag>/templates/commands/clarify.md`) each of `templates/commands/{specify,plan,tasks,clarify,analyze,constitution,checklist,implement,converge,taskstoissues}.md`, the document skeletons `templates/{spec,plan,tasks,constitution,checklist}-template.md`, and the shared automation `scripts/bash/common.sh`. All of these are small enough to read in a single fetch each.
- The extension/preset/bundle/workflow implementation behind the catalog CLI (`src/specify_cli/extensions/__init__.py`, `_commands.py`, `agents.py`, `catalogs.py`, `shared_infra.py`, `integration_status.py`) is much larger (tens of KB up to ~140KB) — don't read these whole. Only reach for them, via a targeted WebFetch prompt asking what specific CLI-invisible behavior a file implements, if something surfaced elsewhere (README, release notes, a template file) points at them specifically.
- Apply the same discipline as 3a's local-code read: capture not just "does this concept exist upstream" but implementation details that make the comparison concrete (e.g. a workflow stage a template expects, a document field a command fills in) — don't just skim for command names.

#### 3d. Diff CLI surface + source surface, pull release narrative

Compare the freshly captured CLI tree against the previous `design/memory/speckit-cli-surface.md` (if present) to list new/changed commands, subcommands, and flags. Also diff 3c's source-derived findings against that same file's source-surface section (see 3f) from the last full analysis. Also WebFetch the release body/notes for the newly-fetched tag (from the same GitHub Releases API response in step 2) and, if present, the repo's `CHANGELOG.md`, for narrative "what changed and why" context.

#### 3e. Compare against the TUI

Cross-reference the CLI-surface diff, the source-derived findings, and the release narrative against `tui-feature-analysis.md`. Flag, in priority order:

1. Anything closing the two known out-of-scope gaps (catalog search/browse of not-yet-installed extensions/presets/workflows; CLI self-check/self-upgrade — note upstream already has a `specify self` command for the latter).
2. Any other net-new CLI surface (new subcommand, new flag on an existing subcommand) not reflected anywhere in `crates/spectatui-core/src/speckit/{cli.rs,registry.rs,workflow.rs}`.
3. Slash-command/template-level features from 3c with no counterpart in Spectatui's `speckit` module or UI. Use judgment here rather than flagging reflexively: commands an AI agent executes directly (e.g. `/speckit.clarify`, `/speckit.analyze`, `/speckit.implement`) may be legitimately out of scope for a TUI that orchestrates spec-kit projects rather than acting as the agent itself — note the distinction explicitly instead of listing every template as "missing". `crates/spectatui-core/src/speckit/workflow.rs`'s `WorkflowStage` enum is the concrete place this kind of comparison already lands (it infers Specified/Clarified/Planned/TasksGenerated/Analyzed/Implementing/Implemented from artifact files) — check whether newly-read templates imply stages or fields that enum doesn't yet track.
4. Anything in the CLI or source surface that already has TUI coverage — note as "already implemented" for completeness.

#### 3f. Report and persist

Prepend to the top of `sync-log.md`'s body (newest first):

```markdown
## <date> — sync #<N> (version <speckit_remote_version> → <new tag>)

### CLI Surface Changes

- <new/changed commands, subcommands, flags>

### Upstream Source Features

- <new/changed slash-command templates, document skeletons, or shared scripts found in 3c>

### Feature Comparison

| Feature | TUI Status                      | Notes |
| ------- | ------------------------------- | ----- |
| ...     | Implemented / Missing / Partial | ...   |

### Recommendations

1. <concrete recommendation, referencing exact files to touch>
```

If this run was forced despite no actual version change (`<speckit_remote_version>` == `<new tag>`), say so explicitly in the header — e.g. `(version v0.12.11 → v0.12.11, forced — user override, no version delta)` — rather than leaving an unexplained `X → X` arrow that reads like a copy-paste mistake. If the CLI surface came back byte-identical to the cached `speckit-cli-surface.md`, say that plainly too instead of leaving "CLI Surface Changes" looking accidentally empty. Same for "Upstream Source Features" if 3c turned up nothing new.

Rewrite `sync-log.md`'s frontmatter: update `last_synced_at`, increment `total_syncs`, and set **both** `speckit_local_version` and `speckit_remote_version` to the tag just analyzed (they always move together, only on a completed full analysis — this is a no-op write when forced with no version delta, which is fine).

Overwrite `design/memory/speckit-cli-surface.md` with the newly captured command tree, plus a second section `### Upstream Source-Derived Feature Surface (templates/scripts, beyond --help)` summarizing 3c's findings with the tag-pinned URLs cited as evidence (so the next full analysis has a cached baseline to diff both parts against, not just the CLI tree), plus fresh `version`/`captured_at` frontmatter.

#### 3g. Report to the user

Print that run's full report section (from 3f) to the user in chat.

## Rules

- Never modify anything outside `design/memory/`.
- Never run a mutating `specify` subcommand — `--help` only.
- Always state plainly whether this run was version-gated (skipped) or fully analyzed, and why.
- A patch-only upstream release short-circuits at step 2 — never re-run the TUI analysis, CLI capture, diff, or comparison for it — unless the user explicitly asked to force a re-analysis, in which case honor that override (see step 2).
- `last_speckit_released_version` always updates every run; `speckit_local_version`/`speckit_remote_version` only ever update together, only in step 3f.
