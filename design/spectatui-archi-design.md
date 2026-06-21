# spectatui — Architecture & Design Doc

*Your TUI dashboard for GitHub Spec-Kit*

**spectatui** (`spectatui`) is a tmux-backed, ratatui-rendered control plane and
visualizer for [GitHub Spec Kit](https://github.com/github/spec-kit) —
giving Spec-Driven Development a dashboard: live workflow status, extensions
and presets, the constitution and spec/plan/tasks artifacts, and one pane per
active feature's coding-agent session.

## 1. What Spec Kit actually looks like (grounding facts)

Confirmed from the spec-kit repo, since the design has to match it exactly:

- **Core workflow commands** (run inside the AI agent as slash commands or
  skills): `/speckit.constitution`, `/speckit.specify`, `/speckit.clarify`,
  `/speckit.plan`, `/speckit.tasks`, `/speckit.analyze`, `/speckit.checklist`,
  `/speckit.implement`, `/speckit.taskstoissues`.
- **Project layout** after `specify init` + a few workflow steps:
  ```
  .
  ├── .specify/
  │   ├── memory/constitution.md
  │   ├── scripts/bash/*.sh          (or powershell equivalents)
  │   ├── templates/*.md             (spec/plan/tasks/CLAUDE templates)
  │   ├── templates/overrides/       (project-local template overrides)
  │   ├── extensions/
  │   │   ├── .registry.json         (installed extensions, source of truth for reads)
  │   │   ├── .backup/               (config backups from `extension remove`)
  │   │   └── <extension-id>/
  │   ├── presets/
  │   │   └── <preset-id>/           (registry file expected alongside, verify per CLI version)
  │   ├── extension-catalogs.yml     (project-level extension catalog config)
  │   └── preset-catalogs.yml        (project-level preset catalog config)
  └── specs/
      └── 001-feature-name/
          ├── spec.md
          ├── plan.md
          ├── tasks.md
          ├── research.md
          ├── data-model.md
          ├── quickstart.md
          └── contracts/
  ```
- **Extensions** add new capabilities. Full CLI surface (confirmed from
  Spec Kit docs): `specify extension search [query] [--tag] [--author]
  [--verified]`, `add <name> [--dev] [--from <url>] [--priority <N>]`,
  `remove <name> [--keep-config] [--force]`, `list [--available] [--all]`,
  `info <name>`, `update [<name>]`, `enable <name>` / `disable <name>`,
  `set-priority <name> <priority>`, and `catalog list|add|remove`.
- **Presets** customize the format/terminology of existing commands. Same
  shape of CLI surface: `specify preset search|add|remove|list|info|update|
  enable|disable|set-priority|resolve|catalog *`. `preset resolve <name>`
  is particularly useful — it traces the full resolution stack for a given
  file and shows which source wins.
- Resolution order, highest priority first: project-local overrides
  (`.specify/templates/overrides/`) → installed presets (by priority) →
  installed extensions (by priority) → Spec Kit core (`.specify/templates/`).
- A feature gets its own git branch and `specs/NNN-name/` directory the
  moment `/speckit.specify` runs.
- 30+ agent integrations are supported (Claude Code, Copilot CLI, Gemini,
  Cursor CLI, Codex CLI, etc.) — `specify integration list` enumerates what's
  installed for a given project.

spectatui's job is to make all of this — workflow stage, constitution,
extensions, presets, and the live agent — visible and navigable in one
terminal screen, without re-implementing Spec Kit itself.

## 1.5. Core principle: the Nx Console model

This is the governing design rule, so it's worth stating explicitly: **spectatui
is to `specify` what Nx Console is to `nx`.** Two strictly different zones:

- **`specs/` (spec.md, plan.md, tasks.md, research.md, ...) — read-only,
  always.** spectatui renders these for visualization only. It never writes to
  them, never offers a "remove" or "edit" action on them, and never calls
  `specify` on their behalf. They're the AI agent's and the user's domain;
  spectatui is a window onto them, not an editor.
- **`.specify/extensions/`, `.specify/presets/`, and their catalogs —
  fully interactive, but CLI-mediated only.** Search, view details, install,
  remove, enable/disable, and reprioritize — every one of these is a thin
  wrapper that shells out to the real `specify extension *` / `specify
  preset *` subcommand and shows the result. spectatui never edits
  `.registry.json`, extension directories, or catalog YAML files directly,
  and never re-implements install/removal/priority logic itself. The CLI is
  the only thing that mutates state; spectatui is the dashboard and the
  confirmation layer in front of it, exactly like Nx Console never computes
  a dependency graph itself — it calls `nx graph` and renders the result.

This keeps spectatui safe to update independently of Spec Kit's internal file
formats (which can change between releases) and means every destructive
action goes through the same validation, confirmation prompts, and backup
behavior the CLI already implements (e.g. `extension remove` backs up config
by default unless `--keep-config`/`--force` is passed).

## 2. Feature set (v1)

1. **Feature/session manager** — list every `specs/NNN-name/` feature,
   its current workflow stage, and its tmux session status.
2. **Spec/plan/tasks browser** — rendered markdown viewer for
   `spec.md` / `plan.md` / `tasks.md` / `research.md` / `data-model.md`,
   with `tasks.md` parsed into a checklist (respecting `[P]` parallel
   markers and per-user-story phases).
3. **Constitution viewer** — `.specify/memory/constitution.md`, always
   one keypress away regardless of which feature is selected.
4. **Extensions & presets manager** — search catalogs, view installed/available
   items with full detail, install, remove, enable/disable, and reprioritize —
   all by shelling out to `specify extension *` / `specify preset *` (see §1.5
   and §5). Catalog management (`specify extension catalog *` / `specify
   preset catalog *`) included.
5. **Workflow timeline** — constitution → specify → clarify → plan →
   tasks → analyze → checklist → implement, per feature, inferred from
   which files exist and (optionally) timestamps/git log on them.
6. **Live agent view** — tail of the tmux pane running the agent for the
   selected feature, with a one-key jump to a fully attached session.
7. **Customizable layout** — show/hide panes, rearrange them, switch
   between layout presets; persisted between runs.
8. **Theming** — light / dark / follow-system, persisted.

## 3. High-level architecture

Unchanged from the tmux-integration discussion: a ratatui app (UI layer +
async core engine) talks to a tmux server (one pane per feature session) and
to the filesystem (`.specify/` + `specs/`). See the architecture diagram
earlier in this conversation — it still holds; what's new below is the data
model, the pane/theme system, and how extensions/presets get surfaced.

## 4. Data model

```rust
struct Project {
    root: PathBuf,
    constitution: Option<PathBuf>,       // .specify/memory/constitution.md
    features: Vec<Feature>,
    extensions: Vec<ExtensionInfo>,
    presets: Vec<PresetInfo>,
}

struct Feature {
    id: String,                          // e.g. "001-create-taskify"
    branch: Option<String>,              // git branch, if discoverable
    dir: PathBuf,                        // specs/001-create-taskify/
    artifacts: FeatureArtifacts,
    stage: WorkflowStage,                // derived, not stored
    session: Option<TmuxSession>,
}

struct FeatureArtifacts {
    spec: Option<PathBuf>,
    plan: Option<PathBuf>,
    tasks: Option<PathBuf>,
    research: Option<PathBuf>,
    data_model: Option<PathBuf>,
    quickstart: Option<PathBuf>,
    contracts_dir: Option<PathBuf>,
}

enum WorkflowStage {
    NotStarted,
    Specified,     // spec.md exists
    Clarified,     // spec.md has a Clarifications section
    Planned,       // plan.md exists
    TasksGenerated,// tasks.md exists
    Analyzed,      // analyze report / marker present
    Implementing,  // tasks.md has some [x] but not all
    Implemented,   // tasks.md fully checked off
}

struct ExtensionInfo {
    id: String,
    version: String,
    status: InstallStatus,        // Enabled, Disabled, Available (not installed)
    priority: Option<u8>,         // None if not installed
    command_count: u32,
    source: ExtensionSource,      // Catalog{name}, Dev{path}, Url{url}
}

struct PresetInfo {
    id: String,
    version: String,
    status: InstallStatus,
    priority: Option<u8>,
    template_count: u32,
}

struct IntegrationInfo {
    name: String,             // e.g. "claude-code", "copilot-cli"
    installed: bool,
    version: Option<String>,
}

enum InstallStatus { Enabled, Disabled, Available }

enum ExtensionSource { Catalog(String), Dev(PathBuf), Url(String) }

struct TmuxSession {
    name: String,
    pane_id: String,
    last_snapshot: String,
    status: SessionStatus,     // Running, Idle, Exited(code)
}

struct StatusBarCounts {
    integrations: u32,   // from `specify integration list`, NOT extensions
    extensions: u32,      // ExtensionInfo entries with InstallStatus::Enabled/Disabled
    workflows: u32,        // tentative: active features/sessions — see §6.5 open question
    presets: u32,            // PresetInfo entries with InstallStatus::Enabled/Disabled
}
```

`WorkflowStage` is derived by checking, in order: does `spec.md` exist? does
it contain a `## Clarifications` section? does `plan.md` exist? does
`tasks.md` exist, and what fraction of its checkboxes are `[x]`? Re-validate
this against whatever the *current* spec-kit template headings are when you
scaffold (templates can change between releases) — treat the exact markdown
structure as a thing to confirm from `.specify/templates/*.md` in the
target project, not hardcode blindly. This is read-only inference only —
it never writes anything back to `specs/`.

`ExtensionInfo`/`PresetInfo`, by contrast, are populated from **two
sources**: parse `.specify/extensions/.registry.json` (and the equivalent
preset registry, if the installed Spec Kit version ships one — verify, since
the docs confirm the extensions registry explicitly but presets weren't
spelled out the same way) for a fast, no-subprocess listing of what's
*installed*; call `specify extension list --available` / `specify preset
search` for what's *available but not installed*. The registry file is
read-only input to spectatui's display — never written by spectatui.

## 5. Extensions/presets manager — CLI action model

This is the Nx-Console-shaped part of spectatui. Every mutating action goes
through one place:

```rust
enum CliTarget { Extension, Preset, Integration } // Integration added per §10 item 5 — confirm via checklist before relying on it

enum CliAction {
    Search { target: CliTarget, query: Option<String>, tag: Option<String>, author: Option<String> },
    Info { target: CliTarget, id: String },
    Add { target: CliTarget, id: String, priority: Option<u8>, dev_path: Option<PathBuf>, from_url: Option<String> },
    Remove { target: CliTarget, id: String, keep_config: bool, force: bool },
    Enable { target: CliTarget, id: String },
    Disable { target: CliTarget, id: String },
    SetPriority { target: CliTarget, id: String, priority: u8 },
    Update { target: CliTarget, id: Option<String> },
    Resolve { name: String },                       // preset-only
    CatalogList { target: CliTarget },
    CatalogAdd { target: CliTarget, url: String, name: String, priority: Option<u8>, install_allowed: Option<bool> },
    CatalogRemove { target: CliTarget, name: String },
}

struct CliJob {
    action: CliAction,
    command_line: String,        // exact `specify ...` invocation, shown before running
    status: JobStatus,           // Pending, Running, Succeeded, Failed(String)
    output: String,              // streamed stdout+stderr
}
```

`SpecifyCliClient` (in `speckit/cli.rs`) maps a `CliAction` to the exact
`specify extension|preset <subcommand>` invocation, runs it via
`tokio::process::Command` with piped stdout/stderr, and streams output lines
into `CliJob.output` as they arrive — surfaced live in a dedicated **CLI
output log** pane (see §6), the same way Nx Console shows a live terminal
panel while a generator runs.

**Confirmation flow**, modeled directly on Nx Console's "preview the
command, then run it" pattern:

1. User picks an action in the extensions/presets panel (e.g. select
   "remove" on an installed extension).
2. spectatui builds the `CliAction` and renders the exact resulting command
   line (`specify extension remove my-ext --force`) for the user to review
   — nothing has run yet.
3. For non-destructive actions (`search`, `info`, `list`, `resolve`,
   `catalog list`), run immediately, no confirmation needed.
4. For destructive/mutating actions (`add`, `remove`, `set-priority`,
   `enable`/`disable`, `update`, `catalog add`/`remove`), require an
   explicit confirm keypress before executing. Default to *not* passing
   `--force` so the underlying CLI's own confirmation prompt is the second
   line of defense — only pass `--force` if the user explicitly opts into
   skipping prompts.
5. On completion, refresh the relevant `ExtensionInfo`/`PresetInfo` list
   from the registry/CLI so the UI reflects the new state — don't
   optimistically mutate local state.

No JSON output flag is documented for `extension`/`preset` subcommands as of
this Spec Kit version (only `specify version --features --json` is
confirmed machine-readable) — parse table-formatted stdout for `search` and
`list`, expecting this to need maintenance as CLI output formatting changes
across Spec Kit releases. Treat `.registry.json` as the more stable source
for *currently installed* state, and reserve text parsing for `search`
results that have no local file equivalent.

## 6. Customizable panes

### Model

```rust
enum PaneKind {
    FeatureList,
    SpecBrowser,
    Constitution,
    ExtensionsPresets,
    WorkflowTimeline,
    AgentOutput,
    CliOutputLog,        // live stdout/stderr of the current CliJob
}

struct PaneConfig {
    kind: PaneKind,
    visible: bool,
    order: u8,           // position within its layout slot
    size_pct: u16,        // relative size within its split
}

enum LayoutMode {
    Tabs,                 // one pane visible at a time, switch with keys
    SplitGrid(Vec<Vec<PaneKind>>), // rows of panes, ratatui Layout splits
}

struct LayoutConfig {
    mode: LayoutMode,
    panes: Vec<PaneConfig>,
}
```

### Rearranging panes in a terminal

Terminals don't support drag-and-drop the way a GUI does, so "rearrange"
means:

- **Keyboard-driven reordering**: a `move pane` mode (e.g. `Ctrl+Shift+Left/Right`
  or vim-style `<` `>`) that swaps the focused pane's position in
  `LayoutConfig.panes`, immediately re-rendering the `ratatui::Layout`
  splits.
- **Show/hide toggles**: a single keybinding per pane kind (or a quick
  command palette) flips `visible`, recomputing the grid so hidden panes
  don't reserve space.
- **Resize**: `+`/`-` or mouse drag (ratatui + crossterm support mouse
  events) adjusts `size_pct` on the splitter between two adjacent panes.
- **Layout presets**: ship 2-3 built-in arrangements (e.g. "Overview" -
  feature list + workflow timeline; "Coding" - agent output + spec browser
  side by side; "Audit" - extensions/presets + constitution) selectable with
  a single key, on top of full manual customization.

### Persistence

Store `LayoutConfig`, `ThemeMode` (§7), and `ForceMode` (§10, item 2 —
`NeverForce`/`AlwaysForce` for destructive `specify` actions) in a config
file via the `directories` crate, e.g. `~/.config/spectatui/config.toml`
(XDG on Linux/macOS, `%APPDATA%` on Windows). Project-specific overrides
can live at `.spectatui/config.toml` inside the target repo if the user
wants per-project layouts — this is a spectatui-only file, not part of
Spec Kit's own override stack, to avoid any confusion with `.specify/`.

## 6.5. Screens

Concrete mockups exist for the screens below (rendered earlier in this
conversation) — this section captures the decisions baked into them so
they survive into implementation.

### Dashboard (default screen)

![spectatui dashboard mockup](spectatui-dashboard.svg)

Three always-visible regions plus the persistent chrome described below:

- **Feature list** (left sidebar) — one row per `specs/NNN-name/` feature:
  stage badge (abbreviated `WorkflowStage`), feature id, and a session
  status dot (green = tmux session running, gray = no/idle session).
  Selecting a row drives every other pane on screen.
- **Workflow stepper** (top right) — the full stage sequence
  (`cons → spec → clar → plan → task → anly → impl`) for the *selected*
  feature, completed stages in one color, the current stage visually
  distinct, plus a one-line progress note (e.g. task completion fraction).
- **Agent output tail** (bottom right) — live `capture_pane` text for the
  selected feature's tmux session, with a status indicator (running/idle)
  and the attach keybinding hint.

### Persistent chrome (present on every screen, not just the dashboard)

Two fixed bars sit below the main content area, outside the
user-customizable pane grid (§7 below) — they are not `PaneKind` values and
are not subject to show/hide/reorder, since they're global navigation/status
rather than content panes:

1. **Keybinding hint line** — context-sensitive shortcuts for whatever
   screen/pane currently has focus.
2. **Status bar** — left-aligned counts, each preceded by a small icon, of
   what's installed for the current project: integrations, extensions,
   workflows, presets. Right-aligned: a gear icon opening the Settings
   screen (layout, theme, CLI/tmux preferences — full content TBD in a
   later pass). This bar is always present regardless of which content
   panes are visible, similar to status lines in tools like k9s or lazygit.

**Open question this surfaces**: "integrations" and "extensions" are
*different* concepts in Spec Kit (agent integrations like Claude Code/
Copilot vs. the extension system from §1/§5), so the integrations count
needs its own data source — `specify integration list`, not the extensions
registry. Add a corresponding `speckit/integrations.rs` module (see §9).
"Workflows" is looser still — it most likely should mean *active
features/sessions* rather than a literal Spec Kit object, since Spec Kit
itself doesn't expose a "workflow" entity beyond the fixed stage sequence.
Pin down that definition before wiring the count.

### Status bar popups

Each stat in the status bar (§6.5) is clickable — mouse click via the
crossterm mouse support already planned for pane resizing, plus a
keyboard fallback (single-letter shortcut shown subtly next to each stat,
e.g. `i`/`e`/`w`/`p`) since not every environment running over SSH/tmux
has usable mouse passthrough. Selecting one opens a centered overlay popup
with detail for that category:

| Status bar item | Popup content |
|---|---|
| integrations | Read-only list from `specify integration list` until the verify-at-scaffold-time checklist (§10) confirms add/remove subcommands exist — at that point, full management actions via `CliTarget::Integration` (same `CliJob`/confirm flow as extensions/presets, §10 item 5). |
| extensions | The full Extensions manager from §5 — search, info, add, remove, enable/disable, set-priority, catalog management. |
| workflows | List of active features/sessions (the same data as the sidebar feature list in the dashboard mockup), for quick jump-to without leaving the current screen. |
| presets | The full Presets manager from §5 — same action set as extensions. |

**Important architectural point**: popups aren't a separate UI
implementation. The extensions/presets popup renders the exact same
widget function as the optional `ExtensionsPresets` pane from §6 — only
the framing differs (overlay box vs. docked pane). This avoids maintaining
two copies of the search/add/remove/confirm flow. The same applies to the
workflows popup and the `FeatureList` pane content.

```rust
enum PopupKind { Integrations, Extensions, Presets, Workflows }

struct PopupState {
    kind: PopupKind,
    // Extensions/Presets popups reuse CliJob/CliAction state from §5 directly
}
```

`App` holds a single `active_popup: Option<PopupState>` — only one popup
at a time. `Esc` or a click outside the overlay closes it; everything
underneath stays exactly as it was (selection, scroll position, in-flight
`CliJob`s keep running). Rendering uses ratatui's `Clear` widget plus a
centered `Rect` over the current screen, regardless of which screen was
active when the popup was opened — so triggering it from the dashboard,
the spec browser, or anywhere else behaves identically.

## 7. Theming



```rust
enum ThemeMode { Light, Dark, System }

struct Theme {
    bg: Color,
    fg: Color,
    accent: Color,
    border: Color,
    muted: Color,
    success: Color,
    warning: Color,
    danger: Color,
}
```

- `System` mode: query the terminal/OS preference at startup (the
  `terminal-light` crate can query the terminal's background color via
  OSC queries on most modern terminals; fall back to `Dark` if detection
  fails or isn't supported by the host terminal — many terminals over SSH
  or inside tmux don't answer the query).
- Re-resolve on a manual keybinding (`t` to cycle Light to Dark to System),
  no restart required.
- Map `Theme` fields to ratatui `Style`s once at startup/theme-change, not
  per-frame, to keep rendering cheap.
- tmux itself has no opinion on color theme for the panes it hosts — this
  only affects spectatui's own chrome, not the agent's own TUI when attached.

## 8. Crate choices (updated)

| Concern | Crate | Notes |
|---|---|---|
| TUI rendering | `ratatui` | |
| Terminal backend | `crossterm` | mouse + raw mode |
| Async runtime | `tokio` | |
| tmux control | shell out via `tokio::process::Command`, wrap in `TmuxClient` | |
| File watching | `notify` | watch `.specify/` and `specs/` |
| Markdown | `pulldown-cmark` | render spec/plan/tasks/constitution |
| Task checklist parsing | hand-rolled, regex/line-scan on `tasks.md` | `[P]` markers, `[x]`/`[ ]` checkboxes, phase headers |
| Config | `serde` + `toml` + `directories` | layout, theme, last project |
| System theme detection | `terminal-light` (best-effort) | |
| CLI args | `clap` | `spectatui --project ~/code/foo` |
| Errors | `thiserror` / `anyhow` | |

## 9. Module layout (updated)

```
src/
  main.rs
  app.rs
  event.rs                  // Key, Tick, TmuxChanged, FsChanged, ThemeChanged
  config.rs                 // LayoutConfig, ThemeMode, load/save
  theme.rs                  // Theme resolution incl. system detection
  layout.rs                 // PaneConfig/LayoutMode, ratatui Layout building
  tmux/
    mod.rs
    session.rs
  speckit/
    mod.rs                  // Project/Feature discovery (specs/ — read-only)
    workflow.rs              // WorkflowStage inference (read-only)
    cli.rs                     // SpecifyCliClient: CliAction -> `specify` invocation, streamed output
    registry.rs                  // parse .specify/extensions/.registry.json (+ presets equivalent)
    extensions.rs                  // ExtensionInfo assembly: registry + CLI search/list
    presets.rs                       // PresetInfo assembly: registry + CLI search/list
    integrations.rs                    // agent integrations via `specify integration list` (distinct from extensions)
    tasks_parser.rs                    // tasks.md -> read-only checklist model
    watch.rs                             // notify -> Event (specs/ + .specify/ for external changes)
  ui/
    mod.rs                   // dispatch active panes per LayoutConfig
    feature_list.rs
    spec_browser.rs
    constitution.rs
    extensions_presets.rs      // list + detail + action menu (search/add/remove/enable/...) — shared by pane and popup
    popup.rs                   // PopupState overlay rendering (Clear + centered Rect), dispatches to the relevant pane widget fn
    cli_confirm.rs                // command-preview confirmation modal
    cli_output.rs                   // live CliJob output pane
    workflow_timeline.rs
    agent_output.rs
    statusbar.rs               // persistent chrome: install counts (§6.5) + settings gear, clickable to open popups (§6.5), not a PaneKind
    settings.rs                // gear-icon destination; layout/theme/CLI prefs (content TBD)
    palette.rs                // command palette / layout-preset picker
```

## 9.5. Nx workspace & release tooling

spectatui will be managed inside an Nx monorepo using the
[`@monodon/rust`](https://github.com/Cammisuli/monodon) community plugin,
which adds Cargo/Rust generators, executors, and Nx Release support to an
Nx workspace. This is a tooling layer on top of everything in §8–9, not a
change to it — Cargo is still the actual build system underneath.

### Setup

```
nx add @monodon/rust
```

This registers generators (`@monodon/rust:binary`, `@monodon/rust:library`,
the latter with an optional `--napi` flag for node-addon builds, not
relevant here) and Cargo-wrapping executors for build/test/lint targets —
exact executor names to confirm against the installed plugin version when
scaffolding, the same "verify against real tooling" caveat as the Spec Kit
CLI details elsewhere in this doc.

### Suggested crate split

Splitting into two Nx projects, rather than one flat binary crate, gives
independent test/build caching and a clean seam if a library reuse case
shows up later (e.g. a headless daemon mode):

```
crates/
  spectatui-core/        (lib crate — generated via @monodon/rust:library)
    Cargo.toml
    src/
      speckit/            // §1–§5 — Project/Feature discovery, CLI client, registry parsing
      tmux/                // §6 (architecture) — TmuxClient
      theme.rs               // §7
      layout.rs                // §6
  spectatui/              (bin crate — generated via @monodon/rust:binary)
    Cargo.toml
    src/
      main.rs, app.rs, event.rs, config.rs, ui/   // depends on spectatui-core
```

The module list in §9 maps directly onto this split: everything under
`speckit/` and `tmux/` moves into `spectatui-core`, everything under `ui/`
plus `main.rs`/`app.rs`/`event.rs`/`config.rs` stays in the `spectatui` bin
crate, which depends on `spectatui-core` as a normal Cargo path dependency.

### Day-to-day commands

Once generated, Nx wraps the usual Cargo workflow:

```
nx build spectatui
nx test spectatui-core
nx lint spectatui          # clippy, via the plugin's executor
```

### Release configuration

`nx.json`, scoping releases to the crates directory and opting into legacy
versioning (currently **required** — `@monodon/rust` doesn't yet implement
the `VersionActions` needed for Nx's newer versioning engine):

```json
{
  "release": {
    "projects": ["crates/*"],
    "version": {
      "useLegacyVersioning": true
    }
  }
}
```

First release (always dry-run first):

```
nx release --first-release --dry-run
nx release --first-release
```

This prompts for a semver bump, writes the new version into each crate's
`Cargo.toml`, updates `Cargo.lock`, generates `CHANGELOG.md` entries from
Conventional Commits, and optionally runs `cargo publish` if you confirm
the publish prompt — relevant later if `spectatui-core` is ever split out
as a standalone published crate, optional for the bin crate itself. Future
releases drop `--first-release` and diff against the previous git tag.

**Practical implication**: this adds a Node.js/npm toolchain dependency
alongside the Rust toolchain, since Nx itself is Node-based — worth noting
since the rest of this project otherwise has no JS dependency.

## 10. Resolved decisions

All eight open questions from earlier drafts are now resolved.

1. ~~Multi-project support~~ — **One instance per project.** `spectatui`
   always operates on the single repo it's run from (or pointed at via
   `--project`), matching the "cd into repo, run spectatui" model.
   Simplifies `Project` to a singleton in `App` state — no project
   switcher UI needed.
2. ~~`CliAction::Add`/`Remove` force behavior~~ — **User-configurable,
   default to never-force.** `ForceMode { NeverForce, AlwaysForce }` lives
   in `config.toml` alongside `LayoutConfig`/`ThemeMode` (§6/§7), defaults
   to `NeverForce` so the CLI's own prompt is the safety net out of the
   box, and is toggleable from Settings.
3. ~~Minimum supported terminal~~ — **SSH + nested tmux is the baseline.**
   The architecture already centers on tmux as the backbone (§3, §6.5's
   agent-attach flow), so designing for the constrained case first —
   limited/no mouse passthrough, terminal theme queries that may not
   answer (§7) — means graceful degradation is built in from day one.
   Richer local terminals get the nicer behavior "for free" once detected.
4. ~~Settings screen scope~~ — **Layout, theme, and force-mode only for
   v1.** No tmux/CLI path overrides or polling-interval tuning yet —
   keeps the first cut small and gives §6.5's gear icon a concrete,
   shippable destination. Revisit scope once the rest of the app is real.
5. ~~Integrations popup action set~~ — **Mirrors extensions/presets if
   `specify integration` turns out to support mutation.** Add
   `CliTarget::Integration` to the `CliAction`/`CliTarget` enum in §5
   alongside `Extension`/`Preset` the moment add/remove subcommands are
   confirmed (item 8 below) — same `CliJob`/confirmation flow, no new
   machinery needed. Until confirmed, the popup stays read-only per the
   table in §6.5.
6–8. ~~Presets registry file shape, `specify integration list` output
   shape, and `@monodon/rust` executor/generator names~~ — **Deferred to
   scaffold time, verified once the real tools are installed during
   generation**, rather than guessed now. These are the three items in
   the checklist immediately below — treat them as gating checks for
   build-order step 1 (Nx/crate scaffolding) and step 2
   (`speckit::Project` discovery), not as design decisions to pre-solve.

### Verify-at-scaffold-time checklist

Run these the moment the real Spec Kit CLI and `@monodon/rust` plugin are
installed, before writing the code that depends on them:

- [ ] `ls .specify/presets/` on a real Spec Kit project — does an
      analogous file to `.specify/extensions/.registry.json` exist? If
      not, `presets.rs` (§9) falls back to `specify preset list` only.
- [ ] `specify integration list --help` (and run it) — confirm output
      shape for `IntegrationInfo` parsing, and whether `add`/`remove`
      subcommands exist (resolves item 5 above for real).
- [ ] `nx g @monodon/rust:binary --help` / check the plugin's generated
      `project.json` — confirm actual executor names for build/test/lint
      before writing them into any CI config or docs beyond this one.



## 11. Suggested build order

1. Initialize the Nx workspace, run `nx add @monodon/rust`, and generate
   the `spectatui-core` lib + `spectatui` bin crates from §9.5 — empty
   shells at this point, just establishing the project boundary before any
   real code.
2. `speckit::Project` discovery for `specs/` (read-only data model) — no UI,
   no CLI calls, verify against a real spec-kit project first.
3. Static ratatui shell: feature list + spec/plan/tasks browser (read-only),
   no tmux, no theming yet.
4. `SpecifyCliClient` + `CliAction` for the non-destructive actions first
   (`search`, `info`, `list`, `catalog list`) — get the streamed-output pane
   working before touching anything mutating.
5. Extensions/presets panel wired to registry reads + the read-only CLI
   actions from step 4.
6. Mutating `CliAction`s (`add`, `remove`, `enable`/`disable`,
   `set-priority`, `update`, `catalog add`/`remove`) with the confirmation
   flow from §5.
7. `TmuxClient` + agent output pane, polling `capture_pane`.
8. `LayoutConfig` + pane show/hide/reorder/resize + persistence.
9. `Theme` + light/dark/system + persistence.
10. Workflow timeline view, tying `WorkflowStage` history together visually.
