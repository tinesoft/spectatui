---

description: "Task list template for feature implementation"
---

# Tasks: Catalog Manager

**Propagated**: 2026-07-11 — Added T024-T026 under Phase 3 (User Story 1)
for FR-015/016/017 (add-form prefill, text-editing affordances, scoped
Ctrl+C-clears behavior). This work had already shipped in code ahead of
spec.md/tasks.md documenting it; the tasks are recorded (and marked done)
for traceability.

**Propagated**: 2026-07-14 — Added T027 under Phase 5 (User Story 3) for the
spec's reprioritize/toggle-in-place refinement, scoped to Extension/Preset
catalog kinds.

**Input**: Design documents from `/specs/002-catalog-manager/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-catalog-commands.md, quickstart.md (all present)

**Tests**: Not explicitly requested in spec.md, but Constitution Principle II mandates colocated unit tests for new parsing/branching logic in `spectatui-core` and UI branching in `crates/spectatui` — those tests are folded into the relevant implementation tasks below (matching this project's existing `mod tests` convention) rather than separate `tests/contract`/`tests/integration` files, since this codebase has no such directories.

**Organization**: Tasks are grouped by user story (US1/US2/US3 from spec.md) to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- File paths are exact, repo-relative

## Path Conventions

Existing two-crate Cargo workspace, no new crate/directory:

- Engine: `crates/spectatui-core/src/speckit/{registry,cli}.rs`
- UI/bin: `crates/spectatui/src/{app,main,event}.rs`, `crates/spectatui/src/ui/*.rs`
- Docs: `design/ui/Spectatui.dc.html`, `README.md`

---

## Phase 1: Setup

**Purpose**: Establish a clean baseline before making changes (no new project/crate/dependency needed — this feature adds to the existing workspace).

- [X] T001 Confirm baseline is green: `pnpm nx run-many -t build,test,lint` for `spectatui-core` and `spectatui`, so any failure surfacing later is attributable to this feature's changes, not pre-existing drift.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The engine-layer catalog-source API, app state, and a reachable-but-minimal popup shell that every user story builds on.

**⚠️ CRITICAL**: No user story task can begin until this phase is complete.

- [X] T002 In `crates/spectatui-core/src/speckit/registry.rs`: promote the private `CatalogKind` enum to `pub`, rename it `CatalogTarget` (4 variants: `Extension`/`Preset`/`Integration`/`Workflow`, keep `cli()` method public); promote `CatalogSource` to `pub` and add a `priority: Option<u8>` field; extend `parse_catalog_urls` to capture the numeric priority from dialect A's `"<name> (priority N)"` header (currently matched but discarded) into that field; rename `catalog_urls` to `pub async fn list_catalog_sources(root: &Path, target: CatalogTarget) -> Vec<CatalogSource>`. Add a unit test in the existing `mod tests` block asserting a dialect-A fixture line produces `priority: Some(N)` and a dialect-B fixture line produces `priority: None`.
- [X] T003 In `crates/spectatui-core/src/speckit/cli.rs`: change `CliAction::CatalogList { target }`, `CatalogAdd { target, .. }`, `CatalogRemove { target, .. }` to take `registry::CatalogTarget` instead of `CliTarget`; update `to_command_line()` to call `target.cli()` for these three variants. No other `CliAction` variant changes. Update/add a unit test asserting `to_command_line()` produces `specify integration catalog list` and `specify workflow catalog add <url> <name>` (the two newly-supported kinds), alongside the existing Extension/Preset coverage. *(depends on T002)*
- [X] T004 [P] In `crates/spectatui/src/event.rs`: add `AppEvent::CatalogSourcesLoaded { target: CatalogTarget, sources: Vec<CatalogSource> }`. *(depends on T002 for the imported types; independent file from T003)*
- [X] T005 In `crates/spectatui/src/app.rs`: add `PopupKind::Catalogs`; add a `CatalogSourcesState` struct (one `Vec<CatalogSource>` per kind: extensions/presets/integrations/workflows) and an `App` field `catalog_sources: CatalogSourcesState`; add `cat_tab: CatalogTarget` (default `Extension`), `cat_index: usize`, `cat_add_input: Option<String>`; add `current_catalog_list(&self) -> &[CatalogSource]` (indexes `catalog_sources` by `cat_tab`) and `cat_select_next`/`cat_select_prev` (mirroring `wf_select_next`/`wf_select_prev`, clamped to `list.len().saturating_sub(1)`); add a minimal `Catalogs` arm in `open_popup()` that resets `cat_index`/`cat_add_input` (tab-persistence nuance deferred to US3/T015). Add unit tests for `cat_select_next`/`cat_select_prev` bounds-clamping (including the empty-list case). *(depends on T002, T004)*
- [X] T006 Create `crates/spectatui/src/ui/catalogs.rs`: `draw()`/`draw_list()`/`draw_detail()`/local `centered()` following the `ui/workflows.rs` template exactly; render the kind-tab row (Extensions/Presets/Integrations/Workflows, highlighting `app.cat_tab`) as a static row (not yet interactive — Tab-cycling comes in US3); list pane with selection bar + status dot (install-allowed vs. discovery-only) using `app.current_catalog_list()`/`app.cat_index`, rendering an empty-state message when the current tab's list is empty; detail pane showing name/url/priority/install-allowed status; leave the "Actions" section empty for now (each user story phase adds its own action line). Register a `ClickAction::SelectSource(i)`-style click region per list row via `app.register_click`, mirroring `ui/workflows.rs`, so mouse selection works identically to every other manager popup (Constitution Principle III). *(depends on T005)*
- [X] T007 Register the new module: add `mod catalogs;` in `crates/spectatui/src/ui/mod.rs`; add a `PopupKind::Catalogs => super::catalogs::draw(frame, app),` arm in `crates/spectatui/src/ui/popup.rs`'s dispatcher, next to the `Workflows` arm. *(depends on T006)*
- [X] T008 [P] In `crates/spectatui/src/ui/statusbar.rs`: append a 6th stat tuple — icon `⧉`, count = total across all four `catalog_sources` vecs, label `"catalogs"`, hint `"c"`, click action `ClickAction::OpenPopup(PopupKind::Catalogs)`. *(depends on T005; independent file from T007/T009)*
- [X] T009 In `crates/spectatui/src/app.rs`'s `palette_commands()` (the palette entries actually live here, not `ui/palette.rs`, which only renders them): add a `"Manage Catalogs"` entry (hint `c`) opening `PopupKind::Catalogs` — for now, same as every other entry point (tab-reset behavior refined in US3/T016). *(depends on T005; independent file from T007/T008)*
- [X] T010 In `crates/spectatui/src/main.rs`: at startup, spawn four `list_catalog_sources` calls (one per `CatalogTarget`) alongside the existing catalog-indexing task, sending `AppEvent::CatalogSourcesLoaded` per kind as each resolves; add the corresponding event-handling arm storing into `app.catalog_sources`; add global `KeyCode::Char('c') => app.open_popup(PopupKind::Catalogs)`; add a `PopupKind::Catalogs` arm in the popup key-match supporting only `Esc` (close) and `Up`/`Down`/`j`/`k` (via `cat_select_next`/`cat_select_prev`) for now; rebind the existing dashboard-only Constitution key from `KeyCode::Char('c')` to `KeyCode::Char('C')` in `handle_dashboard_key`; update the "Go to Constitution" palette entry's `hint` from `"c"` to `"C"` (`app.rs`, `PaletteCommand`). Also change `App::poll_cli_job`'s return type from `()` to `Option<CatalogTarget>`: when a completed job succeeds and `self.cli_job.as_ref().map(|j| &j.action)` matches `CliAction::CatalogAdd { target, .. }` or `CliAction::CatalogRemove { target, .. }`, return `Some(*target)` (in addition to still calling the existing `refresh_project()`, which doesn't cover catalog sources). In `main.rs`'s main loop, after `app.poll_cli_job()`, if it returns `Some(target)`, spawn a fresh `list_catalog_sources(root, target)` and send `AppEvent::CatalogSourcesLoaded` — the same spawn shape as T016's manual-refresh path, just triggered by a successful add/remove instead of the `r` key. *(depends on T003, T004, T005, T007, T009)*

**Checkpoint**: The Catalog Manager is reachable via status bar, `c`, and the command palette; it shows real (async-fetched) data for the Extensions tab with working selection movement. Constitution is reachable via `C`. No add/remove/tab-switch/refresh actions exist yet — user story implementation begins now.

---

## Phase 3: User Story 1 - Add a new catalog source (Priority: P1) 🎯 MVP

**Goal**: A user can add a catalog source for the current tab's resource kind, previewing the exact command before it runs.

**Independent Test**: Open the Catalog Manager (Extensions tab, from Foundational), add a source, confirm the preview, and see it appear in the list on success.

### Implementation for User Story 1

- [X] T011 [US1] In `crates/spectatui/src/ui/catalogs.rs`: when `app.cat_add_input.is_some()`, render the inline add-form in place of the tab row (`"Add <kind> catalog:  <input>█"` prompt + hint line below); add the `[a] add source` line to the (now non-empty) Actions list; swap the footer hint to the add-form variant (`type · url name [priority]`, `enter · add`, `esc · cancel`) while adding. *(depends on T006)*
- [X] T012 [US1] In `crates/spectatui/src/main.rs`'s `PopupKind::Catalogs` key arm: handle `a` (set `cat_add_input = Some(String::new())`); while `cat_add_input.is_some()`, handle character input (append), `Backspace` (pop), `Esc` (clear/cancel), and `Enter` (parse the buffer as `"url name [priority]"`, build `CliAction::CatalogAdd { target: cat_tab, url, name, priority }`, clear `cat_add_input`, and route through the existing `request_cli_action`/confirm-popup path used by the other managers; on success this now triggers a `catalog_sources` refresh for `cat_tab` via T010's extended `poll_cli_job`, so no additional refresh logic is needed here). Add a unit test (colocated with the parsing helper, in `app.rs` or `main.rs` as appropriate) covering: well-formed input with and without a priority, and malformed input (missing name). *(depends on T010, T011)*
- [X] T024 [US1] Pre-fill the add form from a selected discovery-only source (FR-015, spec refinement 2026-07-11). In `crates/spectatui/src/app.rs`: add `cat_add_open()`, which opens `cat_add_input` pre-filled as `"url name [priority]"` from `current_catalog_list()[cat_index]` when that source's `install_allowed` is `false`, and empty otherwise (nothing selected, empty list, or an install-allowed source) — sets `cat_add_cursor` to the end of the pre-filled text. In `crates/spectatui/src/main.rs`: change the `a` key handler to call `app.cat_add_open()` instead of setting `cat_add_input = Some(String::new())` directly. Mirror the same prefill logic in `design/ui/Spectatui.dc.html`'s `a`-key handler. Add unit tests in `app.rs`'s `mod tests` covering: prefill without a priority, prefill with a priority, empty when the selected source is already install-allowed, and empty when nothing is selected. *(depends on T011, T012)*
- [X] T025 [US1] Add-form text-editing affordances — cursor movement, delete, mouse click-to-position, and paste (FR-016, spec refinement 2026-07-11). In `crates/spectatui/src/app.rs`: add `cat_add_cursor: usize` (char index, not byte index) to `App`, plus `cat_add_insert_str`/`cat_add_backspace`/`cat_add_delete_forward`/`cat_add_move_left`/`cat_add_move_right`/`cat_add_move_home`/`cat_add_move_end`/`cat_add_set_cursor` helpers, all UTF-8-boundary-safe (operate via `char_indices`, not raw byte offsets). In `crates/spectatui/src/event.rs`: add `AppEvent::Paste(String)`, sourced from `crossterm`'s bracketed-paste events. In `crates/spectatui/src/main.rs`: enable/disable bracketed paste (`EnableBracketedPaste`/`DisableBracketedPaste`) around the terminal session; add `Left`/`Right`/`Home`/`End`/`Delete` handling in the add-form key arm; route `AppEvent::Paste` into `cat_add_insert_str` only while the add-form is open (no-op elsewhere); add a `ClickAction::SetCatalogAddCursor(usize)` variant and its `execute_click_action` handling. In `crates/spectatui/src/ui/catalogs.rs`: rewrite `draw_add_form` to render a scrollable text viewport (reserving one column each side for `‹`/`›` overflow indicators via a new pure `scrolled_visible_range` helper) and register one `ClickAction::SetCatalogAddCursor` click region per visible character. Mirror cursor movement, delete, paste (`onPaste`), and the scrollable viewport render in `design/ui/Spectatui.dc.html`. Update `README.md`'s Catalogs section documenting these affordances. Add unit tests: `app.rs` covering each editing helper (including a UTF-8 multi-byte-character boundary case) and `ui/catalogs.rs`'s `mod tests` covering `scrolled_visible_range`'s scroll-to-keep-cursor-visible behavior at the start/middle/end of the input and when the cursor is clamped beyond the char count. *(depends on T011, T012)*
- [X] T026 [US1] Scope Ctrl+C to clear the add-form's input instead of the app's global quit while the form is open (FR-017, spec refinement 2026-07-11). In `crates/spectatui/src/app.rs`: add `cat_add_clear()` (resets `cat_add_input` to an empty string and `cat_add_cursor` to `0`, leaving the form open — distinct from `Esc`, which cancels the form entirely). In `crates/spectatui/src/main.rs`'s `handle_key`: in the existing Ctrl+C branch, check `app.active_popup == Some(PopupKind::Catalogs) && app.cat_add_input.is_some()` first — if true, call `app.cat_add_clear()` instead of setting `should_quit`; Ctrl+C's global-quit behavior is unchanged everywhere else. Update the add-form hint line (`ui/catalogs.rs` and `design/ui/Spectatui.dc.html`) to include `ctrl+c · clear`. Add a unit test asserting `cat_add_clear` empties the input, resets the cursor, and leaves the form open. *(depends on T011, T012, T025 — reuses `cat_add_cursor`/the add-form hint line T025 introduces)*

**Checkpoint**: User Story 1 is fully functional and independently testable.

---

## Phase 4: User Story 2 - Remove an unwanted catalog source (Priority: P1)

**Goal**: A user can remove a selected catalog source for the current tab's resource kind, previewing the exact command before it runs.

**Independent Test**: Select an existing source, remove it, confirm the preview, and see it disappear from the list on success.

### Implementation for User Story 2

- [X] T013 [US2] In `crates/spectatui/src/ui/catalogs.rs`: add the `[x] remove source` line to the Actions list, shown only when `app.current_catalog_list()` is non-empty (mirrors how other managers hide actions with no valid target). *(depends on T006)*
- [X] T014 [US2] In `crates/spectatui/src/main.rs`'s `PopupKind::Catalogs` key arm: handle `x` when a source is selected — build `CliAction::CatalogRemove { target: cat_tab, name }` and route through the existing confirm-popup path (on success this now triggers a `catalog_sources` refresh for `cat_tab` via T010's extended `poll_cli_job`, so no additional refresh logic is needed here). In `crates/spectatui/src/app.rs`: ensure `cat_index` is clamped whenever the active kind's `Vec<CatalogSource>` shrinks (e.g. after a successful removal or a refresh returning fewer sources) so it never indexes out of bounds; add a unit test for this clamping specifically after a simulated shrink (distinct from T005's general bounds test, which covers navigation, not post-mutation shrink). *(depends on T010, T013)*

**Checkpoint**: User Stories 1 AND 2 both work independently.

---

## Phase 5: User Story 3 - Browse across all four resource kinds and refresh (Priority: P2)

**Goal**: A user can switch between the four resource kinds' tabs and manually refresh the current tab's list; the manager remembers the last-viewed kind across opens except when opened from the command palette (FR-014).

**Independent Test**: Open the manager, switch through all four tabs, refresh one of them, then verify the sticky-vs-reset tab behavior across the three entry points.

### Implementation for User Story 3

- [X] T015 [US3] In `crates/spectatui/src/app.rs`: split popup-opening into two explicit paths — one that preserves `cat_tab` (for status-bar and global-keypress entry) and one that resets it to `CatalogTarget::Extension` (for the command-palette entry) — per FR-014. Add a unit test covering both paths. *(depends on T005)*
- [X] T016 [US3] In `crates/spectatui/src/main.rs`: add `Tab`/`Shift+Tab` handling in the `PopupKind::Catalogs` key arm to cycle `cat_tab` through all four kinds and reset `cat_index`; add `r` handling to re-spawn `list_catalog_sources` for the current `cat_tab` and send the same `AppEvent::CatalogSourcesLoaded` event (real refresh, not a no-op); update the global `c` key handler and, in `crates/spectatui/src/ui/palette.rs`, the "Manage Catalogs" entry's `run` action to call the two respective methods added in T015. Register a click region per kind-tab label (via `app.register_click`) that sets `cat_tab` to that kind and resets `cat_index`, mirroring the Extensions/Presets tab row's mouse support (Constitution Principle III). *(depends on T010, T015)*
- [X] T017 [US3] In `crates/spectatui/src/ui/catalogs.rs`: add the `[r] refresh` line to the Actions list; confirm the tab row (already rendering `app.cat_tab`, static since T006) needs no visual change beyond it now actually responding to `Tab`/`Shift+Tab` from T016. *(depends on T006)*

**Checkpoint**: All three user stories are independently functional; the full feature matches spec.md.

- [X] T027 [US3] Deliver the spec's 2026-07-14 refinement: reprioritize/toggle
  catalog sources in place (previously out of scope), scoped to Extension/Preset
  catalog kinds only. **Result**: verified against the installed `specify 0.12.4`
  CLI that `extension catalog`/`preset catalog` support
  `--priority`/`--install-allowed` on `add`, but `integration catalog`/`workflow
  catalog` do not (no priority/install-allowed concept for those two kinds at
  all) — confirming `registry.rs`'s existing `CatalogSource` "dialect A/B"
  comment and scoping this feature accordingly. No single CLI edit/update verb
  exists (`list`/`add`/`remove` only), so added an `e` key
  (`App::cat_edit_available`/`cat_edit_open` in `crates/spectatui/src/app.rs`,
  gated to Extension/Preset in the `ui/catalogs.rs` Actions list) that opens the
  existing add-form pre-filled with the selected source's current
  url/name/priority/install-allowed regardless of its install-allowed state
  (unlike `cat_add_open`, which only pre-fills discovery-only sources). Submitting
  the edit form dispatches `CatalogRemove` then, once it succeeds, a
  `CatalogAdd` carrying the edited values — chained via a new
  `App::pending_followup_action` field polled in `poll_cli_job`, and aborted
  (not chained) if the remove fails. Added `install_allowed: Option<bool>` to
  `CliAction::CatalogAdd` in `crates/spectatui-core/src/speckit/cli.rs` so the
  re-add can pass `--install-allowed`/`--no-install-allowed` explicitly (`None`
  preserves the existing plain "add a new source" behavior of omitting the flag).

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Constitution-mandated documentation sync (Principle III) and final quality gates (Principles I/II).

- [X] T018 [P] In `design/ui/Spectatui.dc.html`: fix the stale `hint:'c'` on the `{label:'Go to Constitution', ...}` palette entry (~line 1094) to `hint:'C'`; add a global `if(k==='C'){ stop(); this.go('constitution'); return; }` handler in the global-keys block (~line 1406-1412) — the mockup never had a direct keyboard shortcut for Constitution, only the (until-now-stale) palette hint.
- [X] T019 [P] In `README.md`'s **Key bindings** section: move Constitution from the Dashboard table's `c` to a `C` entry; add `c` → "Open Catalogs popup" to the **Global** table; add a new **Catalogs popup** table (`Tab`/`Shift-Tab` switch kind, `↑`/`k`·`↓`/`j` navigate, `a` add, `x` remove, `r` refresh, `Esc` close — explicitly no `/` filter, unlike the other manager popups).
- [X] T020 Run `pnpm nx run-many -t lint` (`cargo clippy --workspace -- -D warnings` equivalent) across both crates; fix any warnings surfaced by this feature's changes.
- [X] T021 Run `pnpm nx run-many -t build` and the format check (`cargo fmt --all -- --check` equivalent); fix formatting.
- [X] T022 Run `pnpm nx run-many -t test` for both crates; confirm all new tests (T002, T003, T005, T012, T014, T015, T024, T025, T026) and all pre-existing tests pass.
- [X] T023 Execute all 8 scenarios in `specs/002-catalog-manager/quickstart.md` manually against a real (or sandboxed) Spec-Kit project, including the `C`/`c` keybinding checks and the add-form prefill/editing/scoped-Ctrl+C checks (scenario 8).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories.
- **User Stories (Phase 3-5)**: All depend on Foundational. US1 and US2 (both P1) have no dependency on each other and can proceed in either order or in parallel; US3 (P2) is independent of US1/US2 (it only extends Foundational's tab/refresh surface) but is sequenced last to match spec.md priority order.
- **T024-T026** (US1, added by the 2026-07-11 spec refinement) all extend the add-form state/UI/key-handling T011/T012 built; T026 additionally depends on T025 for the `cat_add_cursor` field and the shared hint line it introduces.
- **Polish (Phase 6)**: Depends on all three user stories being complete.

### Within Foundational

T002 → T003 (cli.rs needs the renamed/`pub` `CatalogTarget`)
T002 → T004 (event.rs needs the `pub` types)
T002, T004 → T005 (app.rs stores `CatalogSource`s and needs the event shape)
T005 → T006 → T007
T005 → T008, T009 (parallel with each other and with T007)
T003, T004, T005, T007, T009 → T010

### Parallel Opportunities

- T004 can run in parallel with T003 (both depend only on T002, touch different files).
- T008 and T009 can run in parallel with each other and with T007 (all depend only on T005/T006, touch different files).
- T018 and T019 can run in parallel (docs, independent files).
- US1 (T011-T012) and US2 (T013-T014) touch overlapping files (`ui/catalogs.rs`, `main.rs`'s same key-match arm) — treat as sequential per story, but the two *stories* can be assigned to different developers working from the same Foundational checkpoint if merged carefully (both extend, rather than conflict with, the same arm).

---

## Parallel Example: Foundational Phase

```bash
# After T002 lands:
Task: "cli.rs — retarget Catalog* actions to CatalogTarget"       # T003
Task: "event.rs — add AppEvent::CatalogSourcesLoaded"             # T004

# After T005/T006 land:
Task: "statusbar.rs — add catalogs stat"                          # T008
Task: "palette.rs — add Manage Catalogs entry"                    # T009
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (critical — blocks everything).
3. Complete Phase 3: User Story 1 (add a source).
4. **Stop and validate**: run quickstart.md scenarios 1-3 against a real `specify` install.
5. This alone delivers the core value: a user can start pulling from a new catalog source without a terminal.

### Incremental Delivery

1. Setup + Foundational → popup reachable, read-only.
2. + US1 → add works (MVP).
3. + US2 → remove works.
4. + US3 → full kind-switching + refresh, matching the design mockup exactly.
5. + Polish → docs in sync, all quality gates green.

---

## Notes

- [P] tasks touch different files with no incomplete dependency.
- Every task names its exact file(s) — no task should require guessing a path.
- Constitution Principle II tests are folded into implementation tasks (T002, T003, T005, T012, T014, T015, T024, T025, T026) rather than split into separate contract/integration test files, matching this project's actual `mod tests` convention.
- Constitution Principle III requires `design/ui/Spectatui.dc.html` and `README.md` updates in the same change (T018-T019, and T024-T026 for the add-form refinements), plus mouse-click parity for list rows and the kind-tab row (T006, T016) — none of this is optional polish to be deferred to a follow-up PR.
- Avoid: editing `specs/`/`.specify/` registry or catalog config files directly anywhere in this feature — every mutation goes through `CliAction`/`SpecifyCliClient`, per architecture doc §1.5.
