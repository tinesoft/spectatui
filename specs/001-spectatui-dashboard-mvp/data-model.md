# Phase 1 Data Model: Spectatui Dashboard — Initial Version

Entities as implemented in `crates/spectatui-core/src/speckit/{mod.rs,workflow.rs,registry.rs}`,
`crates/spectatui-core/src/tmux/mod.rs`, and `crates/spectatui/src/config.rs`. Each maps
directly to a Key Entity in `spec.md`. All fields are read from disk or a subprocess and
are never written back to `specs/` (spec FR-002, FR-012) except `AppConfig`, which is the
one entity spectatui itself persists.

## Project

The single Spec-Kit-managed repository spectatui is pointed at. One instance per running
process (spec Assumption: no multi-project switching).

| Field | Type | Notes |
|---|---|---|
| `root` | path | Repository root (from `--project`, default `.`) |
| `constitution` | optional path | `.specify/memory/constitution.md`, if present |
| `features` | list of `Feature` | Discovered from `specs/NNN-name/` |
| `extensions` | list of `ExtensionInfo` | Installed + available |
| `presets` | list of `PresetInfo` | Installed + available |
| `integrations` | list of `IntegrationInfo` | Installed + available |
| `workflows` | list of `WorkflowInfo` | Installed; populated asynchronously after initial discovery |

**Validation**: `root` must exist; absence of `.specify/` is a degraded state (spec Edge
Cases), not a construction error — `Project` still constructs with empty collections so
the UI can render the "not a recognized Spec-Kit project" message.

## Feature

One `specs/NNN-name/` unit of work.

| Field | Type | Notes |
|---|---|---|
| `id` | string | e.g. `"001-spectatui-dashboard-mvp"` |
| `branch` | optional string | Associated git branch, if discoverable |
| `dir` | path | `specs/NNN-name/` |
| `artifacts` | `FeatureArtifacts` | Which documents exist |
| `stage` | `WorkflowStage` | **Derived**, not stored — recomputed on every discovery/refresh |

**Relationships**: owned by exactly one `Project`; owns exactly one `FeatureArtifacts` and
derives exactly one `WorkflowStage`.

## FeatureArtifacts

| Field | Type |
|---|---|
| `spec` | optional path |
| `plan` | optional path |
| `tasks` | optional path |
| `research` | optional path |
| `data_model` | optional path |
| `quickstart` | optional path |
| `contracts_dir` | optional path |

**Validation**: every field is independently optional; the browser (spec FR-010) must
render "not yet created" per-tab rather than treating any combination as invalid.

## WorkflowStage (state machine, read-only inference)

```
NotStarted → Specified → Clarified → Planned → TasksGenerated → Analyzed → Implementing → Implemented
```

Plus an out-of-band **Unknown** result (spec Clarifications: unrecognized template
conventions) that does not participate in the linear sequence — it is a terminal
degraded-display state, not a stage.

**Transition rule** (evaluated fresh on every discovery, never cached across a file
change): does `spec.md` exist → does it contain a Clarifications section → does `plan.md`
exist → does `tasks.md` exist → what fraction of its checkboxes are checked (0% = generated
but not started counts as `TasksGenerated`; partial = `Implementing`; 100% = `Implemented`)
→ is an analysis marker present. Each check is independent file/content inspection; no
state is persisted between runs.

**Progress note**: `tasks.md` progress is a single flat `(done, total)` checkbox count
across the whole file for v1 (spec Assumption — no per-user-story-phase breakdown).

## ExtensionInfo / PresetInfo

| Field | Type | Notes |
|---|---|---|
| `id` | string | |
| `version` | string | |
| `status` | `InstallStatus` (`Enabled` \| `Disabled` \| `Available`) | `Available` = catalog-known, not installed |
| `priority` | optional u8 | `None` iff not installed |
| `command_count` (ext) / `template_count` (preset) | u32 | |
| `source` | `ExtensionSource` (`Catalog(name)` \| `Dev(path)` \| `Url` \| `Local`) — extensions only | |
| `author` | optional string | |
| `description` | string | |

**Validation**: `priority` MUST be `None` when `status == Available` (an uninstalled item
has no priority) and `Some` otherwise.

## IntegrationInfo

| Field | Type | Notes |
|---|---|---|
| `key` | string | e.g. `"claude"` |
| `name` | string | Display name |
| `installed` | bool | |
| `is_default` | bool | Matches `integration.json`'s `default_integration` |
| `cli_required` | bool | CLI tool vs. IDE-only |
| `version` | optional string | |
| `description` | string | |

**Validation**: at most one `IntegrationInfo` in a `Project` may have `is_default == true`.

## WorkflowInfo

| Field | Type | Notes |
|---|---|---|
| `id` | string | e.g. `"speckit"` |
| `name` | optional string | e.g. `"Full SDD Cycle"` |
| `version` | optional string | |
| `source` | optional string | e.g. `"bundled"`, `"catalog · community"` |
| `installed` | bool | |
| `description` | string | |
| `last_run` | optional string | Run-history summary |

**Note**: distinct from `WorkflowStage` — this is Spec-Kit's separate "automation
pipeline" concept (spec Key Entities: Automation Workflow), not the lifecycle sequence.

## Coding-Agent Session (`TmuxSession` / `SessionStatus`)

| Field | Type | Notes |
|---|---|---|
| `name` | string | tmux session name |
| `pane_id` | string | |
| `status` | `SessionStatus` (`Running` \| `Idle` \| `Exited` \| `NotFound`) | v1 only ever produces `Running`/`Idle` (spec Assumption) |
| `last_snapshot` | list of lines | Captured pane output for the tail view |

## CliAction / CliJob (the mutation contract — see also `contracts/`)

| Field | Type | Notes |
|---|---|---|
| `action` | `CliAction` | Which `specify …` operation |
| `command_line` | string | Exact command previewed to the user before execution (FR-019) |
| `status` | `JobStatus` (`Pending` \| `Running` \| `Succeeded` \| `Failed`) | |
| `output` | string | Streamed stdout+stderr |

**Invariant** (spec FR-019a): the app holds at most one `CliJob` at a time
(`Option<CliJob>`, not a list/queue) — a new action cannot start while `status ==
Running`.

## User Preferences (`AppConfig`)

| Field | Type | Notes |
|---|---|---|
| `theme` | string (`"dark"` \| `"light"`) | |
| `accent` | string (`"indigo"` \| `"teal"` \| `"amber"`) | |
| `dashboard_layout` | string (`"overview"` \| `"coding"` \| `"audit"` \| implicit `"custom"` when `custom_layout` is active) | |
| `mouse_support` | bool | |
| `agent_tail_follow` | bool | |
| `confirm_before_force` | bool | Governs whether destructive actions default to skipping the CLI's own `--force` prompt |
| `tmux_prefix` | string | Session-naming prefix |
| `config_location` | string | Which of the resolvable config paths is active (read-only display, FR-028) |
| `custom_layout` | optional `CustomLayout` | Present iff the user has built one |

**Persistence**: read from a project-local override file first, else a fixed fallback
chain of user-level locations (see `research.md`); written back on every settings change.

## CustomLayout / PaneConfig

| Field | Type | Notes |
|---|---|---|
| `panes` | ordered list of `PaneConfig` | |

`PaneConfig`: `kind` (`PaneKind`: `FeatureList` \| `SpecBrowser` \| `Constitution` \|
`ExtensionsPresets` \| `WorkflowTimeline` \| `AgentOutput` \| `CliOutputLog`), `visible`
(bool), `order` (u8, position within layout), `size` (u8, 1–4, relative size within its
split).

**Validation**: `order` values determine render sequence among `visible == true` panes
only; hidden panes retain their `order`/`size` so re-showing them restores their prior
position (spec FR-024 acceptance scenario 2).
