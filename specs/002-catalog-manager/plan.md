# Implementation Plan: Catalog Manager

**Branch**: `002-catalog-manager` | **Date**: 2026-07-10 | **Spec**: [spec.md](./spec.md)

**Propagated**: 2026-07-11 — Reflected spec.md's FR-015/016/017 (add-form
prefill from a selected discovery-only source, text-editing affordances,
and scoped Ctrl+C-clears behavior) into Project Structure below; this work
had already shipped in code ahead of the spec/plan documenting it.

**Propagated**: 2026-07-14 — Reflected spec.md's 2026-07-14 refinement:
reprioritize/toggle-in-place is delivered for extension/preset catalog
sources via a new `e` edit action that sequences a real `CatalogRemove` then
`CatalogAdd` (now carrying an explicit `install_allowed: Option<bool>` field)
behind one confirm step — no new `CliAction` variant was needed since the
underlying tool still has no dedicated edit/update verb.

**Input**: Feature specification from `/specs/002-catalog-manager/spec.md`

## Summary

Add the unified **Catalog Manager** popup described in
`design/core/spectatui-archi-design.md` §2 item 12 / §5 / §6.5: a single,
kind-tabbed (Extensions/Presets/Integrations/Workflows) view of catalog
*sources*, supporting add/remove/refresh, each delegated to
`specify <kind> catalog *` with the same preview-then-confirm flow the other
four resource managers already use. Most of the engine-layer plumbing for
this already exists but is unused — `CliAction::CatalogList/Add/Remove` in
`cli.rs` and the private `CatalogKind`/`CatalogSource`/catalog-URL-scraping
code in `registry.rs` were pre-anticipated but never wired to any UI. This
feature promotes that code to a public, 4-kind-wide surface and builds the
popup, status-bar stat, keybindings, and command-palette entry on top of it,
following the same structural template as the existing `ui/workflows.rs`
popup. It also resolves the one keybinding conflict this surfaces: the
Constitution viewer's existing dashboard-only `c` moves to `C` so that global
`c` can open Catalogs, matching how the other four managers each get a
global letter (`i`/`f`/`w`/`e`/`p`).

## Technical Context

**Language/Version**: Rust 1.75, edition 2021 (workspace `rust-version`)

**Primary Dependencies**: `ratatui` 0.29 + `crossterm` 0.28 (UI/terminal,
`crates/spectatui`), `tokio` (async runtime, both crates), `serde`/`serde_json`
(registry/config parsing), `reqwest` (rustls-tls, catalog JSON fetch),
`notify`/`notify-debouncer-mini` (fs watch, unaffected by this feature),
`thiserror`/`anyhow` (errors) — no new dependency is introduced by this
feature; it reuses the existing crate graph (§8 of the architecture doc).

**Storage**: N/A — catalog source state is owned entirely by the external
`specify` CLI/registries; spectatui never persists or caches it beyond an
in-memory, refresh-on-demand snapshot (consistent with the Nx-Console-style
"never mutate state directly" principle in §1.5 of the architecture doc).

**Testing**: `cargo test --workspace` via `nx run-many -t test` (constitution
Principle II); new unit tests colocated in `mod tests` blocks in
`registry.rs` (priority parsing) and `app.rs` (selection/tab-state
transitions), matching the existing convention.

**Target Platform**: Cross-platform terminal (Linux/macOS/Windows via
`crossterm`) — no platform-specific work needed; this feature only adds a
popup and key/event wiring, no new platform surface.

**Project Type**: Two-crate Cargo workspace (`spectatui-core` lib +
`spectatui` bin) inside an Nx-managed monorepo (§9/§9.5) — this feature adds
to both crates, no new crate.

**Performance Goals**: Constitution Principle IV — the render/input loop
must not block; keypresses must be reflected within one 100ms input-poll
tick. Catalog-source fetching (initial load and manual refresh) MUST run as
a spawned `tokio` task and communicate back via the existing `AppEvent`
channel, exactly like the current startup catalog-indexing task — never
inline on the render path.

**Constraints**: `cargo clippy --workspace -- -D warnings` and
`cargo fmt --all -- --check` must pass (Principle I); no new `.unwrap()`/
`.expect()` outside tests; every add/remove action must go through the
existing preview-then-confirm `CliConfirm`/`CliOutput` popup flow, never
mutate registry/config files directly (Principle III + architecture §1.5).

**Scale/Scope**: Small — four resource kinds, each typically holding a
handful of catalog sources (the existing design mockup ships 2 sources per
kind as a representative example); this is a single-user local desktop tool,
not a scale-sensitive system.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|---|---|---|
| I. Code Quality | New code goes through `cargo clippy`/`fmt`; CLI process spawning reuses the existing `SpecifyCliClient::spawn_job` (already returns `Result`-shaped events, no new `.unwrap()` needed); any `#[allow(...)]` (e.g. removing the stale `#[allow(dead_code)]` on `CatalogSource::install_allowed` once it's rendered) is a *removal*, not a new suppression. | **PASS** |
| II. Testing Standards | New parsing logic (priority extraction in `parse_catalog_urls`) gets a unit test in `registry.rs`'s existing `mod tests`; new branching UI state (tab cycling, add-form vs. list-view rendering) gets unit tests in `app.rs`, consistent with existing coverage for `wf_select_next`/`wf_select_prev`-style methods. | **PASS** |
| III. User Experience Consistency | `design/ui/Spectatui.dc.html` already has the Catalogs popup fully worked out (it's ahead of the code here) — this feature is *catching the code up to the mockup*, plus a small mockup fix of its own (the stale `hint:'c'` on the "Go to Constitution" palette entry, and adding the missing direct `Shift+C` handler, which the mockup never actually had). Both the mockup and `README.md`'s Key bindings table need updating **as part of this feature**, not after — see Project Structure below. New popup follows the identical dark/light + 3-accent theming already used by every other popup (no hardcoded colors). | **PASS, with an explicit task to update `design/ui/Spectatui.dc.html` and `README.md` in the same change** |
| IV. Performance Requirements | Catalog-source list fetch/refresh is async via `tokio::spawn`, results delivered through the existing `AppEvent` enum/channel — same shape as the current startup indexing task; no blocking calls added to the render or input-poll path. | **PASS** |
| V. Conventional Commit Discipline | Not a design-time gate; applies at commit time to whatever commits implement this plan. | **N/A at planning stage** |

No violations requiring a Complexity Tracking entry.

## Project Structure

### Documentation (this feature)

```text
specs/002-catalog-manager/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   └── cli-catalog-commands.md
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

### Source Code (repository root)

This is a two-crate Cargo workspace (Option 1 shape, adapted to the existing
`crates/` layout — not a generic template, the real current tree):

```text
crates/
  spectatui-core/                         (lib — engine)
    src/speckit/
      registry.rs        # CHANGED: CatalogKind → pub CatalogTarget; CatalogSource → pub
                          #   + priority field; catalog_urls → pub list_catalog_sources
      cli.rs              # CHANGED: CatalogList/Add/Remove take CatalogTarget (4 kinds)
                          #   instead of CliTarget (2 kinds)
  spectatui/                               (bin — UI + event loop)
    src/
      event.rs             # CHANGED: + AppEvent::CatalogSourcesLoaded;
                            #   + AppEvent::Paste(String) (FR-016, bracketed paste)
      main.rs               # CHANGED: spawn per-kind source fetch; new PopupKind::Catalogs
                            #   key handling; global `c`; Constitution key `c` → `C`;
                            #   add-form cursor keys (Left/Right/Home/End/Delete, FR-016),
                            #   Paste event routed into the add-form only (FR-016),
                            #   Ctrl+C scoped to clear the add-form input instead of
                            #   global quit while it's open (FR-017)
      app.rs                # CHANGED: + PopupKind::Catalogs, cat_tab/cat_index/
                            #   cat_add_input/catalog_sources state, open_popup() arm,
                            #   selection helpers; palette hint `c` → `C` for Constitution;
                            #   + cat_add_cursor (char index) and cat_add_{insert_str,
                            #   backspace,delete_forward,move_left,move_right,move_home,
                            #   move_end,set_cursor,clear}() editing helpers (FR-016);
                            #   + cat_add_open() prefilling from the selected discovery-only
                            #   source's url/name/priority (FR-015)
      ui/
        catalogs.rs         # NEW: popup draw fn, mirrors ui/workflows.rs template;
                            #   draw_add_form renders a scrollable text viewport with
                            #   `‹`/`›` overflow indicators and a per-char click region
                            #   (ClickAction::SetCatalogAddCursor) so the mouse can
                            #   position the cursor (FR-016)
        mod.rs               # CHANGED: + `mod catalogs;`
        popup.rs             # CHANGED: + PopupKind::Catalogs dispatch arm
        statusbar.rs         # CHANGED: + 6th "catalogs" stat tuple
        palette.rs           # CHANGED: + "Manage Catalogs" entry

design/ui/
  Spectatui.dc.html                        # CHANGED: fix stale Constitution palette
                                            #   hint (c → C) + add the missing Shift+C
                                            #   direct-key handler (mockup never had one);
                                            #   mirror the add-form prefill, cursor/paste
                                            #   editing, and scoped Ctrl+C behavior
                                            #   (FR-015/016/017) so the mockup and app match

README.md                                  # CHANGED: Key bindings tables — move `c`
                                            #   (Constitution) from Dashboard to `C`;
                                            #   add `c` (Catalogs) to Global; add a new
                                            #   Catalogs popup keybindings table; document
                                            #   the add-form's editing affordances and
                                            #   scoped Ctrl+C behavior (FR-015/016/017)
```

**Structure Decision**: No new crate, no new top-level directory. Everything
lands inside the existing two-crate split (§9 of the architecture doc):
engine changes in `spectatui-core::speckit::{registry,cli}`, UI/state/wiring
changes in the `spectatui` bin crate's existing `app.rs`/`main.rs`/`event.rs`
plus one new `ui/catalogs.rs` module following the established per-manager
popup pattern (`ui/workflows.rs`, `ui/integrations.rs`, etc.).

## Complexity Tracking

*No Constitution Check violations — table intentionally omitted.*
