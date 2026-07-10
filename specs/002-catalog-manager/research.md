# Phase 0 Research: Catalog Manager

No items in Technical Context were marked `NEEDS CLARIFICATION` — this
feature's technical shape was fully investigated before planning (via direct
codebase exploration of `crates/spectatui-core/src/speckit/{cli,registry}.rs`,
`crates/spectatui/src/{app,main,event}.rs` and `ui/*.rs`, plus the design
mockup `design/ui/Spectatui.dc.html`). This document consolidates the
resulting decisions.

## Decision: Widen the existing `CliAction::CatalogList/Add/Remove` rather than add new variants

**Rationale**: `crates/spectatui-core/src/speckit/cli.rs` already has
`CliAction::CatalogList { target }`, `CatalogAdd { target, url, name,
priority }`, `CatalogRemove { target, name }` with working
`to_command_line()` and `is_destructive()` handling — but `target` is typed
as `CliTarget` (`Extension`/`Preset` only). Meanwhile
`crates/spectatui-core/src/speckit/registry.rs` already has a private
4-variant `CatalogKind { Extension, Preset, Integration, Workflow }` used
internally to scrape `specify <kind> catalog list` for the "available items"
pipeline. Promoting `CatalogKind` to `pub` (renamed `CatalogTarget` to match
the architecture doc's naming) and re-pointing the three `CliAction` variants
at it is a small, non-duplicative change that reuses code already proven to
produce correct command lines for all four kinds.

**Alternatives considered**: Introducing a brand-new `CatalogTarget` enum in
`cli.rs` with its own `Extension|Preset|Integration|Workflow` variants and a
manual mapping to/from `registry::CatalogKind` — rejected as pure duplication
with no benefit; the two enums would need to stay in lockstep forever.

## Decision: Reuse `catalog_urls`/`parse_catalog_urls`, extended to capture priority

**Rationale**: `registry.rs` already fetches and parses the exact CLI output
this feature needs to display (`CatalogSource { name, url, install_allowed
}`), just as a private implementation detail. Dialect A's output line
(`"<name> (priority N)"`) already contains the priority number; the current
parser matches the `"priority"` keyword to decide install-allowed status but
discards the digit. Extending it to capture that digit into a new
`priority: Option<u8>` field on a now-`pub` `CatalogSource` is a minimal,
low-risk extension of existing, already-tested parsing logic.

**Alternatives considered**: Calling a hypothetical JSON-emitting catalog
list command — rejected; the architecture doc's own CLI-surface grounding
facts (§5) state only `specify version --features --json` and `specify
integration status --json` are genuinely machine-readable. `catalog list`
has no such flag, so text scraping (already implemented) is the only option.

## Decision: New popup follows the `ui/workflows.rs` structural template exactly

**Rationale**: Every existing manager popup (`integrations.rs`,
`workflows.rs`, `extensions.rs`/`presets.rs`) shares one draw pattern:
centered dimmed-backdrop rect → title with count → list pane (~40-46% width,
selection bar + status dot) → vertical divider → detail pane (header +
status + description + "Actions" list of `[key] label` rows) → footer hint.
`workflows.rs` is the closest precedent (single resource kind, no internal
tabs) and is the base template; the Catalogs popup adds one differentiator
(the kind-tab row / inline add-form, matching the mockup) but nothing
structurally new. This keeps the feature visually and architecturally
indistinguishable from the app's existing popups, per Constitution
Principle III.

**Alternatives considered**: A bespoke layout for Catalogs — rejected; would
introduce a one-off interaction pattern the constitution explicitly warns
against ("Error and confirmation states... MUST follow the same visual and
interaction pattern already established, rather than introducing a one-off
pattern per feature" generalizes directly to popup layout).

## Decision: Constitution keybinding moves from `c` (dashboard-only) to `C` (Shift+C, global)

**Rationale**: User decision (recorded during planning, 2026-07-10) — `c` is
needed globally for Catalogs, matching how every other resource manager
(Integrations `i`, Features `f`, Workflows `w`, Extensions `e`, Presets `p`)
already gets a single global letter per the README's existing Key Bindings
table. Rebinding Constitution to `C` follows the app's own established
`t`/`T` (theme/accent) convention for a related pair of actions sharing a
letter at different cases, so it introduces no new binding style.

**Alternatives considered**: Giving Catalogs a non-mnemonic letter instead
(e.g. `g`) — rejected by the user in favor of matching the architecture
doc's explicit design (§2 item 12, §6.5) which specifies `c` for catalogs
throughout (status bar hint, global key, palette hint).

## Decision: Catalog-kind tab selection is sticky, except from the command palette

**Rationale**: Resolved during `/speckit-clarify` (Session 2026-07-10, see
`spec.md` FR-014): opening via the status-bar stat or the global `c` key
preserves whichever kind was last viewed; opening via the command palette
always resets to Extensions. This exactly matches the design mockup's
existing (and already-implemented) distinction between
`openCatalogs(this.state.catTab||'extensions')` (status bar / global key) and
`openCatalogs('extensions')` (palette entry) — the mockup had already made
this call; the spec clarification just made it an explicit, testable
requirement instead of an implicit mockup detail.

**Alternatives considered**: Always resetting to Extensions regardless of
entry point (simpler, but loses continuity for the most-used entry points
and contradicts the mockup, which is the UI source of truth per Constitution
Principle III).

## Decision: No reprioritize or install-allowed/discovery-only toggle actions

**Rationale**: The real `specify <kind> catalog *` CLI surface is
`list|add|remove` only (architecture doc §1 grounding facts). Neither the
`CliAction` enum nor the design mockup's interactive Catalogs popup
implements a set-priority or toggle verb for catalog sources — the mockup's
"Actions" list is exactly `add source` / `remove source` / `refresh`.
Reprioritizing is therefore achieved by removing a source and re-adding it
with a different `--priority` value (which `CatalogAdd` already accepts),
not a new dedicated action. `install_allowed`/discovery-only is
display-only, reflecting whatever the underlying tool reports.

**Alternatives considered**: Inventing a `CliAction::CatalogSetPriority` /
`CatalogToggleInstall` pair speculatively — rejected as scope creep with no
underlying CLI command to back it; would be dead code identical in kind to
the very problem this feature is fixing (unused, speculative `CliAction`
variants).
