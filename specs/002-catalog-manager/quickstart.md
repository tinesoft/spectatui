# Quickstart: Validating the Catalog Manager

## Prerequisites

- Rust toolchain per `rust-version = '1.75'` (both crates).
- Nx workspace deps installed (`pnpm install` at the repo root, if not
  already done).
- A Spec-Kit project directory to point spectatui at (any directory with a
  `.specify/` folder — the workspace root itself qualifies, since it has
  `.specify/extensions.yml`, `.specify/templates/`, etc.).
- Optional but recommended for the full add/remove flow: the real `specify`
  CLI installed and on `PATH`. Without it, `list_catalog_sources` returns an
  empty list per kind (see `contracts/cli-catalog-commands.md`'s failure
  behavior) and add/remove attempts will show a failed-command CLI Output
  popup — this is correct, expected behavior in a sandbox without `specify`,
  not a bug to chase.

## Build

```sh
pnpm nx build spectatui-core
pnpm nx build spectatui
```

## Run

```sh
pnpm nx run spectatui:run -- --project .
# or, once built:
./target/debug/spectatui --project .
```

## Validation scenarios (map to spec.md User Stories)

1. **Reach the Catalog Manager three ways** (FR-010):
   - Click the "catalogs" stat in the status bar.
   - Press `c` from the Dashboard (and confirm it also works from at least
     one other screen, e.g. Spec Browser, since it's a global binding).
   - Open the command palette (`:` or `Ctrl-K`) and run "Manage Catalogs".

2. **Tab-persistence clarification (FR-014)**: switch to the Workflows tab,
   close the popup (`Esc`), reopen via `c` — expect it reopens on Workflows.
   Then open the command palette's "Manage Catalogs" entry — expect it
   opens on Extensions regardless of what was last viewed.

3. **User Story 1 — add a source**: on any tab, press `a`, type
   `<url> <name> [priority]`, press `Enter`. Expect the confirm popup to show
   the exact `specify <kind> catalog add ...` command. Confirm, and expect
   the CLI Output popup to stream the result; on success the new source
   should appear in the list after the popup closes/list refreshes.

4. **User Story 2 — remove a source**: select an existing source, press `x`,
   confirm. Expect the same preview-then-confirm flow, and the source gone
   from the list on success.

5. **User Story 3 — browse + refresh**: `Tab`/`Shift-Tab` through all four
   kinds; press `r` on one of them and confirm the list is re-fetched (watch
   for a request against `specify <kind> catalog list` if you have a way to
   observe subprocess calls, e.g. `strace`/`dtrace`/a wrapper script).

6. **Constitution keybinding preserved (SC-005)**: press `C` (Shift+C) from
   the Dashboard — expect the Constitution viewer opens. Confirm plain `c`
   no longer opens it (it now opens Catalogs instead).

7. **Docs updated per Constitution Principle III**: confirm
   `design/ui/Spectatui.dc.html` reflects the `C`/`c` split (open it in a
   browser, check the command palette's "Go to Constitution" hint and try
   the `Shift+C` key), and that `README.md`'s Key Bindings tables list `C`
   under Dashboard/Global for Constitution, `c` under Global for Catalogs,
   and a new Catalogs-popup table (tab/↑↓/a/x/r/esc — no `/` filter).

8. **Add-form prefill, editing, and scoped Ctrl+C (FR-015/016/017)**:
   - Select a discovery-only source (one not install-allowed), press `a` —
     expect the add form to open pre-filled with that source's
     `url name [priority]`, cursor at the end. Select an install-allowed
     source (or nothing) and press `a` — expect the form opens empty.
   - While the form is open, type some text, then use `←`/`→`/`Home`/`End`
     to move the cursor, `Delete` to remove the character ahead of it, and
     `Backspace` to remove the one behind it — confirm the cursor and text
     update as expected, including on a string long enough to scroll (watch
     for the `‹`/`›` overflow indicators). Click partway through the visible
     text with the mouse and confirm the cursor jumps there.
   - Paste text (bracketed paste from a terminal, or a clipboard paste in
     the browser mockup) — expect it inserted at the cursor, with no
     newlines carried over even if the pasted text contained any.
   - Press `Ctrl-C` while the form is open — expect the input clears but the
     form stays open (not an app quit); confirm `Ctrl-C` still quits the app
     everywhere else (e.g. from the Dashboard, or with the form closed).

## Automated checks

```sh
pnpm nx test spectatui-core   # includes new priority-parsing unit test
pnpm nx test spectatui        # includes new App selection/tab-state tests
pnpm nx lint spectatui-core spectatui
```

All four must be clean (Constitution Principles I/II) before this feature is
considered done.
