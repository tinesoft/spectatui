# Quickstart: Validating the Spectatui Dashboard (Initial Version)

Validation guide for the capability set described in `spec.md`. See `data-model.md` for
entity shapes and `contracts/` for the CLI-action and config-file contracts referenced
below — not duplicated here.

## Prerequisites

- Rust toolchain per `rust-version` in `Cargo.toml` (MSRV 1.75; dev container ships 1.96.0)
- Node/pnpm for the Nx-orchestrated tasks (`pnpm install` once, per `CONTRIBUTING.md`)
- `tmux` installed and on `PATH` (for User Story 1's session/attach scenarios)
- The `specify` CLI installed and on `PATH` (for User Story 3's manager scenarios)
- A sample Spec-Kit project with at least one feature in `specs/NNN-name/` — this repo
  itself qualifies (it now has `specs/001-spectatui-dashboard-mvp/`)

## Build & unit tests

```sh
pnpm nx run-many -t test    # cargo test --workspace, per Constitution Principle II
pnpm nx run-many -t lint    # cargo clippy --workspace -- -D warnings
pnpm nx build spectatui     # cargo build -p spectatui
```

Expected: all pass with zero warnings before any manual validation below.

## Launching against a sample project

```sh
cargo run -p spectatui -- --project .
# or, against a different Spec-Kit project:
cargo run -p spectatui -- --project /path/to/other-speckit-project --theme light --accent teal
```

## Validation scenarios (map to `spec.md` User Stories)

### US1 — Feature/lifecycle monitoring + live agent (P1)

1. Start spectatui against a project with 2+ features in different lifecycle stages.
   **Expect**: the feature list shows every feature with a stage badge and a running/idle
   dot (distinct shape per FR-004, not color-only).
2. Select a feature with no running session, focus the Agent pane (`Tab`), and press
   `Enter`. **Expect**: spectatui creates a tmux session named `<tmux_prefix><feature id>`
   (see `tmux_prefix` in `contracts/config-schema.md`) running the default coding agent,
   and hands off full-screen to it immediately.
3. Detach (`Ctrl-b d`, or the session's own detach key) to return to the dashboard, then
   press the attach keybinding (`a`) on the same feature. **Expect**: full-screen handoff
   back to that same tmux session; detaching again returns to the dashboard cleanly.
4. While spectatui is running, externally edit that feature's `plan.md`. **Expect**: the
   lifecycle badge updates within the fs-watch debounce window (~500ms) with no manual
   refresh (SC-005).

### US2 — Artifact & constitution browsing (P2)

1. Select a feature with `spec.md`/`plan.md`/`tasks.md`/`research.md` all present, open
   the artifact browser, and tab through each. **Expect**: rendered headings/lists/bold
   text, and `tasks.md` checkboxes with `[P]` markers visually distinguished (FR-009).
2. Select a feature missing one artifact type. **Expect**: that tab clearly shows "not
   created" rather than an empty/blank pane (FR-010).
3. From any screen, open the constitution viewer. **Expect**: same
   `.specify/memory/constitution.md` content regardless of prior feature selection.

### US3 — Extensions/presets/integrations/workflows management (P3)

Cross-reference `contracts/specify-cli-actions.md` for the exact command each action maps to.

1. Open the Extensions manager, select an installed extension, trigger Remove.
   **Expect**: the exact `specify extension remove <id> ...` command line is shown before
   anything runs; nothing changes until confirmed (FR-019).
2. Confirm the action. **Expect**: live streamed output, then the list refreshes from the
   registry — not an optimistic in-memory update (FR-021, FR-022).
3. While that action is running, attempt to trigger a second action (e.g., disable a
   different extension). **Expect**: blocked/disabled until the first completes
   (FR-019a).
4. Open the Integrations manager and request `status` on an installed integration.
   **Expect**: drift detail is shown (or a clean "no drift" result).
5. Open the Workflows manager and run the bundled `speckit` workflow. **Expect**: run
   status becomes visible without leaving the dashboard, and no enable/disable/priority
   controls are offered for it (FR-018).

### US4 — Layout & theming persistence (P4)

1. Press `1`/`2`/`3` to switch dashboard layouts. **Expect**: immediate visual change per
   layout (Overview/Coding/Audit).
2. Enter the layout editor, hide a pane, reorder two panes, resize one, exit.
   **Expect**: `Custom` layout reflects all three changes immediately.
3. Toggle theme (`t`) and cycle accent (`T`). **Expect**: consistent recoloring across
   every currently visible screen/popup.
4. Quit and relaunch with no CLI overrides. **Expect**: layout, theme, and accent are
   exactly as left (SC-004) — verify against the schema in `contracts/config-schema.md`.

### US5 — Keyboard/mouse navigation (P5)

1. Using only the keyboard, reach every screen, open every manager popup via its
   status-bar fallback letter, and quit with confirmation.
2. Open the command palette (`:` or `Ctrl+K`), type a substring of a command name.
   **Expect**: the list filters to matching entries (substring match, not fuzzy — per
   spec Assumptions).
3. With mouse support enabled, repeat a subset of the above via clicks. **Expect**:
   identical results to the keyboard equivalents. Disable mouse support and confirm
   clicks now have no effect.

## Edge-case spot checks

- Run against a directory with no `.specify/` — expect a clear "not a recognized
  Spec-Kit project" state, not a crash or blank screen.
- Run with `tmux` not on `PATH` — expect session indicators to show "unavailable"
  degraded state (spec Edge Cases).
- Trigger a mutating action while the `specify` binary is not on `PATH` — expect a
  visible, readable failure in the CLI output log, with no list falsely updated.
