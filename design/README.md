# Handoff: spectatui — TUI dashboard for GitHub Spec-Kit

## Overview
spectatui is a tmux-backed, **ratatui**-rendered control plane for GitHub Spec-Kit
(see `spectatui-archi-design.md`). This package is the **visual + interaction
spec** for the v1 screens: dashboard, extensions/presets manager, spec/plan/tasks
browser, constitution viewer, status-bar popups, command palette, settings, the
pane layout editor, and the full-screen session-attach handoff.

## About the design files
`spectatui — TUI for Spec-Kit.html` (standalone, opens in any browser) and
`spectatui.dc.html` (source) are a **design reference**, not code to ship. The
mock is an HTML simulation of a terminal: it renders a fixed **132 cols × 40 rows**
character buffer with real box-drawing glyphs to preview exactly how the real
ratatui app should look. **Recreate it natively in Rust with `ratatui` +
`crossterm`** following the module layout and data model already specified in
`spectatui-archi-design.md` (§4, §6, §9). The HTML's layout math, palette, and
keymap below are the source of truth for the *look*; the arch doc is the source
of truth for the *architecture*.

> The mock's 132×40 grid is just a fixed preview canvas. The real app uses the
> live terminal size — treat all pixel/cell numbers below as **ratio + minimum**
> guidance expressed via `ratatui::layout::Constraint`, not hardcoded sizes.

## Fidelity
**High-fidelity.** Final colors, glyphs, borders, spacing rhythm, badge styling,
and the complete keymap are all decided. Match them.

---

## Design tokens → ratatui

All colors are `ratatui::style::Color::Rgb(r, g, b)`. Map each theme field once at
startup / theme-change into a `Theme` struct of `Style`s (arch doc §7), not
per-frame.

### Dark theme (default)
| Token | Hex | Use |
|---|---|---|
| `bg` | `#101013` | terminal background |
| `panel` | `#16161b` | pane fill, status bar |
| `panelAlt` | `#1c1c23` | active tab fill, inline code |
| `fg` | `#d7d7dc` | primary text |
| `dim` | `#8b8b95` | secondary text |
| `faint` | `#5b5b65` | tertiary / disabled / hints |
| `border` | `#2c2c35` | unfocused pane border |
| `sel` | `#23232e` | selected-row background |
| `selFg` | `#f0f0f5` | selected-row text |
| `good` | `#84d48f` | running, success, done steps |
| `warn` | `#e3b673` | in-progress, `--force`, `[P]` markers |
| `bad` | `#ef8c7d` | failure, destructive |
| `info` | `#7cc2e8` | filenames, branches, counts |
| `headerBg` | `#0b0b0e` | top header bar |

### Light theme
| Token | Hex |
|---|---|
| `bg` `#f4f1ea` · `panel` `#fbf9f4` · `panelAlt` `#efece4` |
| `fg` `#2c2a26` · `dim` `#76726a` · `faint` `#a8a397` |
| `border` `#ddd8cc` · `sel` `#e8e3d6` · `selFg` `#1b1a17` |
| `good` `#2f8a3f` · `warn` `#a9701a` · `bad` `#cb5341` · `info` `#2f76a8` · `headerBg` `#ebe7dd` |

### Accent palette (cycle with `T`) — `(dark, light)`
| Name | Dark | Light |
|---|---|---|
| indigo (default) | `#93a4ff` | `#5159d4` |
| teal | `#5fd6bf` | `#1c9685` |
| amber | `#e6b552` | `#b07414` |

Accent drives: focused pane border, focused title (bold), selection left-bar
(`▌`), tab highlight, progress-bar fill, header brand word.

### Workflow stage badges — `fg` on `bg`, rendered as ` cons ` (space-padded pill)
| Stage | Dark fg / bg | Light fg / bg |
|---|---|---|
| `cons` | `#7fd8c2` / `#0f3a32` | `#0b6052` / `#cfeee6` |
| `spec` | `#f0a47c` / `#43210f` | `#a5512b` / `#f6ddcf` |
| `clar` | `#e8a6c5` / `#3d1b2d` | `#9c3a68` / `#f4d9e6` |
| `plan` | `#f0c879` / `#42330c` | `#8a6310` / `#f3e7c4` |
| `task` | `#86b8ec` / `#10314f` | `#2d6ba3` / `#d6e7f6` |
| `anly` | `#c4a7f0` / `#2c1f47` | `#5d3aa0` / `#e7ddf6` |
| `impl` | `#90d890` / `#123a1d` | `#2f8a3f` / `#d4eed9` |

Stage order: `cons → spec → clar → plan → task → anly → impl`. In the workflow
stepper, **completed** steps use `good` fg on bg `#16271b` (dark) / `#dcefdf`
(light); the **current** step uses its own stage badge + bold; **pending** steps
are `faint`. Separator between steps: `─►` in `faint`.

### Glyphs (single-width; the mock font is JetBrains Mono — the real app inherits the user's terminal font, which must include box-drawing + these symbols)
- Borders: **rounded** — `╭ ╮ ╰ ╯ ─ │` → `Block::default().border_type(BorderType::Rounded)`.
- Title in border: `┤ Title ├` inset 2 cols from the left corner.
- Status dots: `●` running/enabled (good), `○` idle/available (faint), `◐` disabled (warn).
- Selection left bar: `▌` in accent on the `sel` background.
- Section marker (headings, phases): `▍ ` in accent.
- Task checkboxes: `[✓]` (good) / `[ ]` (faint); parallel marker `[P]` (warn).
- Progress bar: `█` filled (accent) / `░` track; e.g. `[████░░░] 9/14 64%`.
- Scrollbar: track `│` (border), thumb `┃` (accent).
- Status-bar icons: `◈` integrations · `◰` extensions · `◷` workflows · `≣` presets · `⚙` settings.
- Tool-call markers (attach view): `⏺` (accent) header, `⎿` (faint) result line.

---

## Screens / views

Persistent chrome on every screen except **attach** (arch doc §6.5):
- **Row 0 — header**: `spectatui ›  taskify  ~/code/taskify` left (brand word in accent+bold); right shows current screen name + theme + accent. Fill `headerBg`.
- **Row h-3 — keybinding hint line**: context-sensitive `key`(accent,bold) + `description`(dim), separated by ` · `(faint).
- **Row h-2 — status bar**: fill `panel`. Left = the four counts each as `icon n label key`; right = `Settings ⚙`. Every stat is clickable and has a single-letter hotkey (`i e w p`) → opens its popup.

### 1. Dashboard (`PaneKind` grid, 3 presets)
Layout via nested `Layout::horizontal`/`vertical` with `Constraint`s:
- **Overview** (`1`): left sidebar **Feature list** `Constraint::Length(38)`; right column split `Constraint::Length(13)` **Workflow** over `Constraint::Min(0)` **Agent output**.
- **Coding** (`2`): **Spec browser** | **Agent output**, 50/50 (`Percentage(50)` each).
- **Audit** (`3`): **Extensions/Presets** `Percentage(54)` | **Constitution** `Min(0)`.
- **Custom** (`4`): user-built in the layout editor (below).

**Feature list** — one feature per 2 rows: row 1 = ` stage ` badge + `id`(selFg/fg) + status dot (right); row 2 = `note`(dim). Selected row: `sel` bg across both rows + accent `▌`. Footer: `+ new feature  [n]`.

**Workflow stepper** — title `Workflow · <id>`; row of stage chips with `─►` separators; then `Current stage: <badge> <verb>`, a `Tasks [bar] x/y pct` line (only when the feature has tasks), and `branch <name>`(info). Height-aware: drop the lower lines first when the pane is short, never overflow the border.

**Agent output** — title `Agent · <agent>`; running/idle dot+label at top-right; live `capture_pane` tail (newest at bottom); footer `[a] attach  [r] refresh tail  [k] kill`. Clip tail to pane height.

### 2. Extensions & Presets manager (arch doc §5, §6)
Two tabs `Extensions n` / `Presets n` (`Tab` switches). Left = list: status dot, id, `pN` priority, `vX.Y.Z`. Right = detail: id+version, status line, `by <author> · <source>`, command/template count, wrapped description, and an **Actions** block whose entries depend on install status (`a` add for available; `x` remove, `e`/`d` enable/disable, `p` set-priority for installed; `r` resolve for presets). This same widget renders both as a dashboard pane and inside the popup — **one function, two frames** (arch doc §6.5).

**CLI action flow** (the Nx-Console pattern, arch doc §5): action → **confirm modal** previewing the exact `specify …` command (`f` toggles `--force`) → on confirm, stream stdout/stderr into a **CLI output** overlay line-by-line with a spinner → on completion show `✓ succeeded · list refreshed` / `✗ failed`, then refresh from registry. Never optimistically mutate.

### 3. Spec / plan / tasks / research browser
Narrow feature sidebar (`Length(30)`) + tabbed document pane (`spec.md` / `plan.md` / `tasks.md` / `research.md`). Markdown rendered with `pulldown-cmark`: `h1`→fg bold, `h2`→`▍ ` accent + bold, body→dim, `• ` bullets accent, code→info indented. `tasks.md` is special: phase headers `▍`, `[✓]`/`[ ]` checkboxes, `[P]` parallel markers, monospaced task ids in info. Right-edge scrollbar; `↑↓`/`j k` scroll; `Tab` switches doc.

### 4. Constitution viewer
Same document renderer, full width, fed `.specify/memory/constitution.md`. Reachable from anywhere.

### 5. Status-bar popups (arch doc §6.5)
Centered overlay = `Clear` widget + centered `Rect`, over a dimmed backdrop (blend every underlying cell's fg ~45% toward bg). Kinds: **integrations** (now a **full manager** mirroring extensions/presets — list + detail + `[a] add` / `[x] remove` / `[u] update`, routed through the same `CliJob`/confirm→CLI flow via `CliTarget::Integration`, mapping to `specify integration add|remove|update <name>`; arch doc §10.5. If the verify-at-scaffold checklist finds the CLI lacks add/remove, degrade this one popup to read-only), **workflows** (active features, `Enter` jumps), **extensions/presets** (the §5 manager widget). `Esc` closes; underlying screen/state is untouched.

### 6. Command palette
`:` (or `Ctrl-K`) opens it. Filter input + filtered command list; `↑↓` select, `Enter` run, type to filter, `Esc` close. Commands cover navigation, layout presets, theme/accent, and the popups.

### 7. Settings
Key/value rows; `↑↓` move, `←→`/`Enter` cycle option or trigger an action row. **v1 scope is fixed (arch doc §10.4): layout, theme, and force-mode only.** Rows: theme, accent, dashboard layout (incl. custom), **Force mode** (`never-force` / `always-force` → `ForceMode` enum, arch doc §10.2, default `never-force`, seeds the confirm modal's initial `--force`), a **Customize panes** action row → layout editor, and a read-only config path. The tmux/agent-tail/mouse prefs and the attach-session row shown in earlier drafts were dropped per §10.4 — do not add them in v1. Persist via `serde`+`toml`+`directories` (arch doc §6, §7).

### 8. Pane layout editor (arch doc §6 "Rearranging panes")
Left = pane list with `◉`/`○` visibility, order index, and a `size ▰▰▱▱` meter. Right = **live wireframe preview** of the resulting grid. Keys: `space` show/hide, `< >` reorder, `+ -` resize (1–4), `Enter` apply (sets the **Custom** layout), `Esc` back. The tiler: first visible pane = left sidebar (`Length(38)`), remaining panes stack in the right column with heights proportional to `size`, each clamped to a per-kind **minimum** (workflow ≥11, agent ≥8, others ≥7–9) so content never gets crushed; when minimums don't all fit, fall back to proportional and let the per-pane clip keep borders clean.

### 9. Session-attach handoff (full-screen)
`a` on the dashboard. Replaces all chrome. Top bar: `● attached  <id> · <agent> · tmux %N` + `Ctrl-b d detach · esc back`. Body = live agent transcript (prompt, prose, `⏺ Verb target +adds -dels` tool calls with `⎿` result lines, a `✻` thinking spinner). Input box near the bottom; footer status line (`agent · model · tokens · branch* · edit/test counts`) + `● session live in tmux`. `Esc` returns to the dashboard; **detach keeps the tmux session running** (arch doc §6, feature 6). In the real app this is literally `tmux attach`/`capture_pane` against the feature's pane — the mock fakes the transcript.

---

## Complete keymap
| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | cycle pane focus (dashboard) / switch document tab (spec) / switch ext↔presets |
| `↑ ↓` / `j k` | move selection / scroll |
| `← →` | prev/next feature (spec) · cycle option (settings) |
| `Enter` | open / activate / confirm |
| `Esc` | back · close popup · detach |
| `1` `2` `3` `4` | dashboard layout: overview / coding / audit / custom |
| `t` | cycle theme (dark ↔ light; arch doc adds System) |
| `T` | cycle accent palette |
| `:` / `Ctrl-K` | command palette |
| `?` | help overlay |
| `i` `w` | integrations / workflows popups |
| `e` `p` | extensions screen / presets popup (dashboard) |
| `a` | attach session |
| `q` | quit (confirm overlay; sessions keep running) |
| **Extensions** | `a` add · `x` remove · `e`/`d` enable/disable · `p` set-priority · `r` resolve · `/` search |
| **Confirm modal** | `Enter` run · `f` toggle `--force` · `Esc` cancel |
| **Layout editor** | `space` show/hide · `< >` reorder · `+ -` size · `Enter` apply |

Mouse: status-bar stats and feature rows are clickable (crossterm mouse events,
same support used for pane-resize). Always provide the keyboard fallback.

---

## State → arch doc data model
The mock's state maps directly onto the arch doc structs:
`screen` + `layout` → `LayoutConfig` (§6); `feat`/selection → `Project.features`
(§4); `extTab`/`extSel` → `ExtensionInfo`/`PresetInfo` lists (§4–5);
`popup` → `App.active_popup: Option<PopupState>` (§6.5);
`paneCfg` → `Vec<PaneConfig>` (§6); `theme`/`accent` → `ThemeMode` + `Theme` (§7);
`forceMode` → `ForceMode { NeverForce, AlwaysForce }` (§10.2, default NeverForce);
confirm/CLI overlay → `CliJob`/`CliAction` (§5, `CliTarget` now also has `Integration`).
`feat`/selection → `Project.features` (§4) — **singleton, one project per instance (§10.1), no switcher.**
Stage is **derived read-only** from which artifacts exist — never written back to `specs/`.

> Open questions are now resolved (arch doc §10): single-project, never-force default, SSH+nested-tmux baseline, v1 settings = layout/theme/force-mode. Three items stay gated on the **verify-at-scaffold checklist** (presets registry shape, `specify integration list` output + add/remove support, `@monodon/rust` executor names) — run those before the code that depends on them.

## Build order (from arch doc §11 — start here)
1. Nx workspace + `@monodon/rust`; generate `spectatui-core` lib + `spectatui` bin.
2. `speckit::Project` discovery for `specs/` (read-only) — verify against a real repo.
3. Static ratatui shell: feature list + spec/tasks browser (this package's screens 1 & 3), no tmux/theming.
4. `SpecifyCliClient` + non-destructive `CliAction`s with the streamed-output pane.
5. Extensions/presets panel on registry reads + read-only CLI.
6. Mutating `CliAction`s + the confirm flow (screen 2 here).
7. `TmuxClient` + agent output + attach (screens 1 & 9).
8. `LayoutConfig` show/hide/reorder/resize + persistence (screen 8).
9. Theme light/dark/system + persistence.
10. Workflow timeline polish.

## Terminal requirements (font & glyphs)

The reference mock sets its own web font (**JetBrains Mono**) and a fixed
**132×40** grid in CSS. A real `ratatui`/`crossterm` TUI cannot set the
terminal's font or point size — that is entirely the user's terminal-emulator
configuration. To reproduce the intended look:

- **Run in a terminal configured with JetBrains Mono** (or another high-coverage
  monospace / Nerd Font). The app inherits whatever font the terminal uses.
- The UI relies on several uncommon Unicode glyphs that render as "tofu" (the
  missing-glyph box) in low-coverage fonts. Confirm the chosen font includes:
  `◈ ❖ ◰ ≣ ◷` (status-bar icons), `▰ ▱` (size meters), `◖ ◗ ◆` (header),
  `◉ ◐ ● ○`, `▌ ▍ █ ░`, and `─►` (stepper arrows).
- The layouts are tuned for ~132 columns; popups clamp on smaller terminals but
  look best at the reference size.

A code-level ASCII fallback for these glyphs is intentionally out of scope for
now; the recommendation above (font choice) is the supported path.

## Files in this bundle
- `spectatui — TUI for Spec-Kit.html` — standalone interactive reference (open in a browser, drive it with the keymap above).
- `spectatui.dc.html` — source of the reference (exact colors/layout/glyph logic live in the embedded script).
- `spectatui-archi-design.md` — the authoritative architecture & data-model doc.

## Kickoff prompt for Claude Code
> Implement spectatui, a ratatui + crossterm TUI for GitHub Spec-Kit, per
> `design_handoff_spectatui/`. Read `spectatui-archi-design.md` for architecture
> and `README.md` for the visual + interaction spec; open the standalone HTML in
> a browser to see the target. Scaffold per arch §9.5/§11 and complete build-order
> step 1 (Nx workspace + `@monodon/rust`, `spectatui-core` lib + `spectatui` bin),
> then step 2 (read-only `speckit::Project` discovery for `specs/`), verifying
> against a real `specify init` project before building UI. Keep `specs/`
> strictly read-only; route every mutation through `specify` subcommands (arch
> §1.5). Match the README's palette, rounded borders, stage badges, and keymap.
