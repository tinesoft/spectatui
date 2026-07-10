# Phase 1 Data Model: Catalog Manager

## Domain entities (from spec.md Key Entities)

### `CatalogTarget` (resource kind)

The four groupings whose catalog sources this feature manages. Promoted from
the existing private `registry::CatalogKind` (no new concept, just a
visibility/name change).

| Variant | CLI noun (`cli()`) |
|---|---|
| `Extension` | `"extension"` |
| `Preset` | `"preset"` |
| `Integration` | `"integration"` |
| `Workflow` | `"workflow"` |

No state transitions — a fixed, closed set, cycled through in the UI via
`Tab`/`Shift+Tab`.

### `CatalogSource`

A named location a `CatalogTarget` resolves installable items from.
Extended from the existing private `registry::CatalogSource`.

| Field | Type | Notes |
|---|---|---|
| `name` | `String` | Display name (e.g. `"default"`, rendered as `"official"` via the existing `catalog_label()` helper; `"community"`) |
| `url` | `String` | Location the source resolves from |
| `priority` | `Option<u8>` | **New field.** `None` if the CLI's `catalog list` output for this source didn't include a priority (dialect B never has one; dialect A does) |
| `install_allowed` | `bool` | `true` = items can be installed from this source; `false` = discovery-only (community catalogs commonly set this) |

**Validation/identity rules**: Uniqueness of `name` within a `CatalogTarget`
is enforced by the underlying `specify` CLI, not by spectatui (per FR-013 —
spectatui never validates or mutates catalog config itself, only delegates
and displays whatever the CLI subsequently reports).

**Lifecycle**: Sources don't have an in-app state machine — they either
exist (returned by `list_catalog_sources`) or don't. Add/remove are
CLI-delegated actions that change what a subsequent `list_catalog_sources`
call returns; spectatui never optimistically mutates its displayed list
(FR-009, edge cases in spec.md).

## Application state (UI layer, `App` in `app.rs`)

Not a "domain" entity in the traditional sense, but this is a TUI feature
where in-memory UI state has real invariants worth capturing:

### `CatalogSourcesState`

```rust
struct CatalogSourcesState {
    extensions: Vec<CatalogSource>,
    presets: Vec<CatalogSource>,
    integrations: Vec<CatalogSource>,
    workflows: Vec<CatalogSource>,
}
```

One `Vec` per `CatalogTarget`, populated independently (each kind's
`AppEvent::CatalogSourcesLoaded` arrives on its own timeline — see
plan.md's async-fetch note). Defaults to all-empty until the first load
completes; an empty `Vec` is a valid, displayable state (spec.md edge case:
"resource kind has zero configured catalog sources").

### New `App` fields

| Field | Type | Invariant |
|---|---|---|
| `cat_tab` | `CatalogTarget` | Which kind's tab is currently shown. Persists across popup closes (sticky), except reset to `Extension` when opened via the command palette (FR-014). |
| `cat_index` | `usize` | Selected row within `catalog_sources.<cat_tab>`. Reset to `0` whenever `cat_tab` changes or the popup is (re)opened. Must be clamped to `list.len().saturating_sub(1)` whenever the underlying list shrinks (e.g. after a successful remove or a refresh that returns fewer sources) so it never indexes out of bounds. |
| `cat_add_input` | `Option<String>` | `None` when not adding; `Some(buffer)` while the inline add-form is active. Cleared on confirm-submit, cancel (`Esc`), or popup close. |
| `catalog_sources` | `CatalogSourcesState` | See above. |

No new persisted config — none of this is written to `AppConfig`/
`config.toml`; it's transient, refetched every app run (and on manual
refresh), consistent with "spectatui never persists catalog source state
beyond an in-memory snapshot" (Technical Context, plan.md).

## Relationships

```text
CatalogTarget (1) ──has many──> CatalogSource (0..n)
App.catalog_sources: one Vec<CatalogSource> per CatalogTarget
App.cat_tab: selects which CatalogTarget's Vec is currently displayed
App.cat_index: selects which CatalogSource within that Vec is currently focused
```
