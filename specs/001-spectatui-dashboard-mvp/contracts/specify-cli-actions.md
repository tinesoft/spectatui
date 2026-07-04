# Contract: `CliAction` → `specify` command line

This is spectatui's primary external interface: every mutating or informational action a
user triggers in an Extensions/Presets/Integrations/Workflows manager maps to exactly one
`specify` CLI invocation, shown to the user verbatim before it runs (spec FR-019). This
table is the contract between `spectatui-core::speckit::cli::CliAction` and the `specify`
binary; a change to either side that breaks this mapping is a contract break.

**Destructive** column marks actions requiring explicit user confirmation (spec FR-019,
`CliAction::is_destructive()`).

## Extension / Preset (`CliTarget`-parameterized: `extension` | `preset`)

| Action | Command line | Destructive | Exposed in UI (v1) |
|---|---|:---:|:---:|
| Search | `specify <target> search [query] [--tag T] [--author A]` | No | **No** (defined, not wired) |
| Info | `specify <target> info <id>` | No | Yes |
| List | `specify <target> list [--available]` | No | **No** (defined, not wired — see note) |
| Add | `specify <target> add <id> [--priority N] [--dev PATH] [--from URL]` | Yes | Yes |
| Remove | `specify <target> remove <id> [--keep-config] [--force]` | Yes | Yes |
| Enable | `specify <target> enable <id>` | Yes | Yes |
| Disable | `specify <target> disable <id>` | Yes | Yes |
| SetPriority | `specify <target> set-priority <id> <priority>` | Yes | Yes |
| Update | `specify <target> update [id]` | Yes | Yes (extensions only — presets have no update) |
| Resolve | `specify preset resolve <name>` | No | Yes (presets only) |
| CatalogList | `specify <target> catalog list` | No | **No** (defined, not wired) |
| CatalogAdd | `specify <target> catalog add <url> <name> [--priority N]` | Yes | **No** (defined, not wired) |
| CatalogRemove | `specify <target> catalog remove <name>` | No¹ | **No** (defined, not wired) |

¹ Not currently flagged destructive by `is_destructive()` despite removing a catalog
source — worth a second look before exposing it in the UI.

## Integrations

| Action | Command line | Destructive | Exposed in UI (v1) |
|---|---|:---:|:---:|
| IntegrationList | `specify integration list` | No | **No** (defined, not wired) |
| IntegrationInstall | `specify integration install <key>` | Yes | Yes |
| IntegrationUninstall | `specify integration uninstall <key>` | Yes | Yes |
| IntegrationUpgrade | `specify integration upgrade [key]` | Yes | Yes |
| IntegrationUseDefault | `specify integration use <key>` | Yes | Yes |
| IntegrationSwitch | `specify integration switch <key>` | Yes | Yes |
| IntegrationStatus | `specify integration status <key> --json` | No | Yes |
| IntegrationGetInfo | `specify integration info <key>` | No | Yes |

## Automation Workflows

| Action | Command line | Destructive | Exposed in UI (v1) |
|---|---|:---:|:---:|
| WorkflowAdd | `specify workflow add <source>` | Yes | Yes |
| WorkflowRemove | `specify workflow remove <id>` | Yes | Yes |
| WorkflowRun | `specify workflow run <source>` | Yes | Yes |
| WorkflowResume | `specify workflow resume <run_id>` | No¹ | Yes |
| WorkflowStatus | `specify workflow status [run_id]` | No | Yes |
| WorkflowGetInfo | `specify workflow info <id>` | No | Yes |
| WorkflowSearch | `specify workflow search [query]` | No | **No** (defined, not wired) |

¹ Resuming a run mutates run state but is not flagged destructive — consistent with it
being a continuation rather than a new/removed installation.

## Self

| Action | Command line | Destructive | Exposed in UI (v1) |
|---|---|:---:|:---:|
| SelfCheck | `specify self check` | No | **No** (defined, not wired) |
| SelfUpgrade | `specify self upgrade` | Yes | **No** (defined, not wired) |

## Contract invariants (enforced by `CliJob` + `App`)

1. **Preview before execute**: `to_command_line()` output MUST be shown to the user before
   any process spawns (spec FR-019).
2. **Single flight**: at most one `CliJob` may have `status == Running` at a time (spec
   FR-019a) — a second action is rejected/blocked at the UI layer, not queued.
3. **No optimistic mutation**: on `JobStatus::Succeeded`, the affected list is re-fetched
   from its source of truth (local registry file or another `specify … list`/`status`
   call) — never patched in memory from assumed success (spec FR-022).
4. **Never bypass the CLI**: spectatui MUST NOT write to `.specify/extensions/`,
   `.specify/presets/`, `integration.json`, or `.specify/workflows/` directly for any of
   the actions above (spec FR-020). The one sanctioned exception is the read-only direct
   catalog-JSON fetch documented in `research.md`, which performs no writes.

## Known gap (tracked, not required for this feature)

`Search`, `List`, `CatalogList/Add/Remove`, `IntegrationList`, `WorkflowSearch`,
`SelfCheck`, and `SelfUpgrade` have complete, tested `to_command_line()`/
`is_destructive()` logic but no UI trigger today. Per spec Assumptions this is
intentional v1 scope (catalog discovery/browsing and self-upgrade are deferred) — this
table exists so a future feature enabling them has zero new engine-side contract work.
