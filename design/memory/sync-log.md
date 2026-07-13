---
last_synced_at: "2026-07-12T00:00:00Z"
speckit_local_version: "v0.12.11"
speckit_remote_version: "v0.12.11"
last_speckit_released_version: "v0.12.11"
total_syncs: 2
---

# Spec-Kit Sync Log

## 2026-07-12 — sync #2 (correction pass, version unchanged at v0.12.11)

**Not a version-triggered sync** — upstream is still `v0.12.11`, unchanged. This is a manual correction pass after evaluating the `sync-with-speckit` skill (running it against forced-reanalysis test prompts in isolated copies) surfaced two content errors in sync #1's report, independent of any version drift. Both were independently re-verified against the real repo (`grep`) before correcting.

### Corrections to sync #1

1. **File misattribution, now fixed.** Sync #1 attributed every `CliAction` construction site (extension/preset add/remove/enable/disable/set-priority/update, integration install/uninstall/switch/use/upgrade/status/info, workflow add/remove/run/resume/status/info, catalog add/remove, preset resolve) to `crates/spectatui/src/app.rs`. It's actually `crates/spectatui/src/main.rs` — same line numbers, wrong file. `app.rs` holds UI/state plumbing (filter helpers, catalog add-form editing, `#[cfg(test)]` unit tests — including the `SelfCheck`/`SelfUpgrade` test-fixture citation, which *was* correctly attributed to `app.rs`), not the dispatch sites themselves.
2. **"Catalog item browse is missing" was wrong.** Sync #1 (and `spec.md:214`) both classified catalog search/browse of not-yet-installed extensions, presets, integrations, and workflows as unbuilt. It's actually implemented — via an async catalog-indexing task in `main.rs:169-172` that fetches each resource's catalog JSON directly (`registry::fetch_available_extensions/presets/integrations`/`fetch_workflows`, `registry.rs:474-574`), bypassing the `specify` CLI's own `search`/`list --available` subcommands (those genuinely remain dead code). Results merge into `Project.*` and render as `available`/`not installed` items in the existing Extensions/Presets/Integrations/Workflows popups, filterable and installable through the existing UI. Full detail in [`tui-feature-analysis.md`](./tui-feature-analysis.md).

### Feature Comparison (corrected)

| Feature | TUI Status | Notes |
| --- | --- | --- |
| Extensions: add/remove/enable/disable/set-priority/update | Implemented | `main.rs:1076-1146` |
| Extensions: catalog item browse (not-yet-installed) | **Implemented** (was "Missing" in sync #1) | `registry::fetch_available_extensions` merged via `app.rs`'s `apply_catalog_cache`, rendered in `ui/extensions.rs:166` |
| Extensions: literal `specify extension search`/`list --available` CLI subcommand | Missing (dead code) | `cli.rs:27-40`, never constructed — TUI reaches the same end result via direct catalog fetch instead |
| Presets: add/remove/enable/disable/set-priority/update/resolve | Implemented | `main.rs:1076-1146` |
| Presets: catalog item browse | **Implemented** (was "Missing") | same mechanism, `ui/presets.rs` |
| Presets: literal `specify preset search` | Missing (dead code) | same pattern |
| Integrations: install/uninstall/switch/use/upgrade/status/info | Implemented | `main.rs:490-549` |
| Integrations: catalog item browse | **Implemented** (was "Missing") | `registry::fetch_available_integrations`, `ui/integrations.rs:219` |
| Integrations: literal `specify integration search` | Missing (dead code) | not constructed anywhere |
| Workflows: add/remove/run/resume/status/info | Implemented | `main.rs:575-613` |
| Workflows: catalog item browse | **Implemented** (was "Missing") | `registry::fetch_workflows`, `ui/workflows.rs` |
| Workflows: literal `specify workflow search` | Missing (dead code) | `cli.rs:127-129` `WorkflowSearch` unwired |
| Catalog sources: list/add/remove (all 4 kinds) | Implemented | `ui/catalogs.rs`, `PopupKind::Catalogs`, `main.rs:656,697` |
| CLI self-check (`specify self check`) | Missing (partially plumbed) | `CliAction::SelfCheck` exists (`cli.rs:130`), only used in `app.rs` test fixtures |
| CLI self-upgrade (`specify self upgrade`) | Missing (partially plumbed) | `CliAction::SelfUpgrade` exists (`cli.rs:131`), same as above |
| Bundles (search/info/list/install/update/remove/validate/build/init/catalog) | Missing entirely | no `CliAction` variant, no popup; net-new upstream primitive |
| Workflow step types (list/add/remove/search/info/catalog) | Missing entirely | net-new upstream primitive under `specify workflow step` |

### Recommendations (updated)

1. **Update `specs/001-spectatui-dashboard-mvp/spec.md:214`.** The "Out of Scope" note is now materially inaccurate: catalog item browse/search for extensions, presets, integrations, and workflows is implemented (via direct catalog-fetch, not the CLI's `search` subcommand). Reword to reflect that only the literal CLI-subcommand path is unused; the end-user capability exists. CLI self-check/self-upgrade remains the one genuinely open item from that note.
2. **Decide whether to route catalog-item discovery through `specify <x> search` instead of raw catalog-JSON fetch**, to avoid drifting from the CLI's own filtering/ranking/auth logic if upstream ever adds behavior the raw JSON can't express. Not urgent — `CliAction::Search`/`List{available:true}`/`WorkflowSearch` already have working `to_command_line()` mappings sitting unused in `cli.rs` if this path is chosen later.
3. **CLI self-check / self-upgrade** — still the cheapest real gap: `CliAction::SelfCheck`/`SelfUpgrade` and their exact command lines already exist in `cli.rs:130-131,318-319`, just unused outside tests. Add a UI entry point (command palette action, or a small section in Settings).
4. **`specify bundle` support** — larger effort, no existing scaffolding. Needs a new `CliAction::Bundle*` variant family, a `PopupKind::Bundles` (or fold into an existing manager), and UI for search/info/list/install/update/remove/validate/build/init, mirroring the pattern used for extensions/presets/workflows in `cli.rs`/`main.rs`/`ui/*.rs`.
5. **`specify workflow step` support** — smaller net-new addition; likely folds naturally into the existing Workflows manager, since step types follow the same list/add/remove/search/info/catalog shape as the other primitives.

## 2026-07-12 — sync #1 (version none → v0.12.11)

**Note (added in sync #2): this entry's Feature Comparison table mis-cited `app.rs` for dispatch code that actually lives in `main.rs`, and incorrectly classified catalog item browse/search as entirely missing. See sync #2 above and [`tui-feature-analysis.md`](./tui-feature-analysis.md) for the corrected findings — this original text is kept for history only.**

First sync ever — no prior `speckit-cli-surface.md` existed to diff against, so this run captured the full CLI surface from scratch and did a ground-truth comparison against Spectatui's actual code (not just its docs). Full details: [`speckit-cli-surface.md`](./speckit-cli-surface.md), [`tui-feature-analysis.md`](./tui-feature-analysis.md).

### CLI Surface Changes

Baseline capture of `specify` v0.12.11. Concepts with no prior lineage in Spectatui's design docs:
- **`specify bundle *`** — a new top-level primitive: discover/install/build/validate versioned, distributable packages composed of extensions/presets/integrations/workflows/step-types.
- **`specify workflow step *`** — manage custom workflow step types (list/add/remove/search/info/catalog).
- **`specify self check` / `specify self upgrade`** — confirmed to exist upstream exactly as spec.md's "Out of Scope" note anticipated.

### Feature Comparison

| Feature | TUI Status | Notes |
| --- | --- | --- |
| Extensions: add/remove/enable/disable/set-priority/update | Implemented | `crates/spectatui/src/app.rs:1076-1146` |
| Extensions: search / list-available (catalog browse) | Missing (dead code exists) | `cli.rs:27-40` defines `Search`/`List{available}`, never constructed anywhere |
| Presets: add/remove/enable/disable/set-priority/update/resolve | Implemented | `app.rs:1076-1146` |
| Presets: search / list-available | Missing (dead code exists) | same pattern as extensions |
| Integrations: install/uninstall/switch/use/upgrade/status/info | Implemented | `app.rs:490-549` |
| Integrations: search | Missing | not wired anywhere |
| Workflows: add/remove/run/resume/status/info | Implemented | `app.rs:575-613` |
| Workflows: search | Missing (dead code exists) | `cli.rs:127-129` `WorkflowSearch` unwired |
| Catalog sources: list/add/remove (all 4 kinds) | Implemented | `ui/catalogs.rs`, `PopupKind::Catalogs`, commit `22e80a7` |
| Catalog **item** browse/search (not-yet-installed contents) | Missing | out of scope per `spec.md:214`, still true today |
| CLI self-check (`specify self check`) | Missing (partially plumbed) | `CliAction::SelfCheck` exists (`cli.rs:130`) but only used in test fixtures |
| CLI self-upgrade (`specify self upgrade`) | Missing (partially plumbed) | `CliAction::SelfUpgrade` exists (`cli.rs:131`), same as above |
| Bundles (search/info/list/install/update/remove/validate/build/init/catalog) | Missing entirely | no `CliAction` variant, no popup; net-new upstream primitive |
| Workflow step types (list/add/remove/search/info/catalog) | Missing entirely | net-new upstream primitive under `specify workflow step` |

### Recommendations

1. **Catalog item browse/search** (closes half of Out-of-Scope gap #1) — lowest-effort win: `CliAction::Search`/`List{available:true}`/`WorkflowSearch` already exist with working `to_command_line()` mappings in `crates/spectatui-core/src/speckit/cli.rs`. Add a results-list view to the existing Extensions/Presets/Workflows popups (`crates/spectatui/src/ui/*.rs`) and a dispatch path in `app.rs` alongside the existing `Add`/`Remove` handlers. No new CLI-invocation logic needed.
2. **CLI self-check / self-upgrade** (closes Out-of-Scope gap #2) — also low-effort: `CliAction::SelfCheck`/`SelfUpgrade` and their exact command lines already exist in `cli.rs:130-131,318-319`, just unused outside tests. Add a UI entry point (command palette action, or a small section in Settings) that constructs and confirms these two actions.
3. **`specify bundle` support** — new, larger effort: no existing scaffolding. Would need a new `CliAction::Bundle*` variant family, a `PopupKind::Bundles` (or fold into an existing manager), and UI for search/info/list/install/update/remove/validate/build/init, mirroring the pattern already used for extensions/presets/workflows in `cli.rs`/`app.rs`/`ui/*.rs`. Lowest priority unless users request bundle-based distribution.
4. **`specify workflow step` support** — smaller net-new addition, likely folds naturally into the existing Workflows manager once catalog-browse (recommendation #1) is built, since step types follow the same list/add/remove/search/info/catalog shape as the other primitives.
5. Update `specs/001-spectatui-dashboard-mvp/spec.md`'s "Out of Scope" note (line 214) to explicitly distinguish catalog *source* management (now implemented) from catalog *item* browse/search (still out of scope) — the current wording reads as if no catalog functionality exists at all.
