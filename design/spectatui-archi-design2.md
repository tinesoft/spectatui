# Spectatui — Architecture & Design Doc

*Your TUI dashboard for GitHub Spec-Kit*

**Spectatui** (`spectatui`) is a tmux-backed, ratatui-rendered control plane and
visualizer for [GitHub Spec Kit](https://github.com/github/spec-kit) —
giving Spec-Driven Development a dashboard: live workflow status, extensions
and presets, the constitution and spec/plan/tasks artifacts, and one pane per
active feature's coding-agent session.

## 1. What Spec Kit actually looks like (grounding facts)

Confirmed from the spec-kit repo, since the design has to match it exactly:

- **Project layout** after `specify init` + a few lifecycle steps:
  ```
  .
  ├── .specify/
  │   ├── memory/constitution.md
  │   ├── scripts/bash/*.sh          (or powershell equivalents)
  │   ├── templates/*.md             (spec/plan/tasks/constitution/checklist templates)
  │   ├── templates/overrides/       (project-local template overrides, only if used)
  │   ├── init-options.json          (recorded init choices: agent, script type, etc.)
  │   ├── integration.json           (current default + installed integrations, per-agent settings)
  │   ├── integrations/
  │   │   ├── claude.manifest.json     (per-integration installed-file hash manifest, for safe upgrade/uninstall)
  │   │   └── speckit.manifest.json    (the core Spec Kit files are tracked as their own "integration" too)
  │   ├── extensions.yml             (install list + hook config, e.g. auto-run after specify/plan)
  │   ├── extension-catalogs.yml     (extension catalog sources, only if customized)
  │   ├── extensions/
  │   │   ├── .registry              (JSON despite no extension — installed extensions, source of truth for reads)
  │   │   └── <extension-id>/
  │   ├── preset-catalogs.yml        (preset catalog sources, only if customized)
  │   ├── presets/
  │   │   ├── .registry              (same shape as extensions/.registry)
  │   │   └── <preset-id>/
  │   └── workflows/
  │       ├── workflow-registry.json   (installed automation workflows — see §1.6)
  │       └── <workflow-id>/workflow.yml
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
- **Top-level `specify` CLI surface** (verified by running
  `uvx --from git+https://github.com/github/spec-kit.git@v0.11.3 specify --help`
  and one level into every subcommand): `init`, `check`, `version
  [--features] [--json]`, `self {check, upgrade}`, `extension`, `integration`,
  `preset`, `workflow`. The last four are the management surfaces this app
  wraps; `init`/`check`/`self` are mostly first-run/maintenance concerns
  (§2 covers where these fit).
- **Slash commands** (run inside the AI agent, installed as Claude Code
  *skills* under `.claude/skills/speckit-*/SKILL.md` for the `claude`
  integration): `/speckit-constitution`, `/speckit-specify`,
  `/speckit-clarify`, `/speckit-plan`, `/speckit-tasks`, `/speckit-analyze`,
  `/speckit-checklist`, `/speckit-implement`, `/speckit-taskstoissues`, and
  `/speckit-converge` (not previously documented here — "assess the
  codebase and append remaining work as tasks"). Internally, manifests and
  registries refer to these with **dots** (`speckit.specify`), but the
  actual invocable name uses the integration's `invoke_separator`
  (`"-"` for Claude, confirmed in `.specify/integration.json`) — other
  agents may differ, worth a quick check per integration if exact command
  strings ever matter in the UI.
- **Extensions** add new capabilities. Full CLI surface (verified):
  `specify extension search [query] [--tag] [--author] [--verified]`,
  `add <name> [--dev] [--from <url>] [--priority <N>]`,
  `remove <name> [--keep-config] [--force]`, `list [--available] [--all]`,
  `info <name>`, `update [<name>]`, `enable <name>` / `disable <name>`,
  `set-priority <name> <priority>`, and `catalog list|add|remove`. A
  real installed extension looks like `agent-context` ("Coding Agent
  Context") — bundled by default, manages `CLAUDE.md`-style context files.
- **Presets** customize the format/terminology of existing commands.
  **Correction from an earlier draft**: presets do **not** have an
  `update` subcommand. Verified surface: `specify preset
  search|add|remove|list|info|set-priority|enable|disable|resolve|catalog
  *`. `preset resolve <name>` traces the full resolution stack for a given
  template and shows which source wins. The default preset catalog
  currently has exactly one installable preset (`lean` — "Lean Workflow");
  many more (23 seen) exist in a discovery-only community catalog,
  installable via `--from <url>` directly rather than `preset add`.
- Resolution order, highest priority first: project-local overrides
  (`.specify/templates/overrides/`) → installed presets (by priority) →
  installed extensions (by priority) → Spec Kit core (`.specify/templates/`).
- A feature gets its own git branch and `specs/NNN-name/` directory the
  moment `/speckit-specify` runs.
- **Integrations** have a meaningfully different shape from
  extensions/presets — not generic add/remove/enable/disable, but
  install-state and "which one is active" semantics: `specify integration
  install|uninstall|switch|upgrade <key>`, `list [--catalog]`,
  `status [--json]`, `use <key>`, `search`, `info <key>`, `scaffold`, and
  `catalog *`. 34 integrations are listed (Claude Code, Copilot, Cursor,
  Codex, Gemini CLI, Windsurf, Zed, and many more), each flagged whether it
  needs a CLI tool vs. is IDE-only, and whether multiple integrations can
  coexist safely in one project (`multi_install_safe`). One project
  typically has one `default_integration` plus zero or more additional
  installed ones. `integration status --json` is genuinely rich and
  machine-readable — includes per-integration file-integrity checking
  (`modified_managed_files`, `missing_files`) useful for a project health
  view, not just a simple installed/not flag.
- **Automation workflows** (`specify workflow`) are a *separate* concept
  from the spec/plan/clarify/.../implement lifecycle — see §1.6.

Spectatui's job is to make all of this — lifecycle stage, constitution,

extensions, presets, and the live agent — visible and navigable in one
terminal screen, without re-implementing Spec Kit itself.

## 1.5. Core principle: the Nx Console model

This is the governing design rule, so it's worth stating explicitly: **Spectatui
is to `specify` what Nx Console is to `nx`.** Two strictly different zones:

- **`specs/` (spec.md, plan.md, tasks.md, research.md, ...) — read-only,
  always.** Spectatui renders these for visualization only. It never writes to
  them, never offers a "remove" or "edit" action on them, and never calls
  `specify` on their behalf. They're the AI agent's and the user's domain;
  Spectatui is a window onto them, not an editor.
- **`.specify/extensions/`, `.specify/presets/`, and their catalogs —
  fully interactive, but CLI-mediated only.** Search, view details, install,
  remove, enable/disable, and reprioritize — every one of these is a thin
  wrapper that shells out to the real `specify extension *` / `specify
  preset *` subcommand and shows the result. Spectatui never edits
  `.registry`, extension directories, or catalog YAML files directly,
  and never re-implements install/removal/priority logic itself. The CLI is
  the only thing that mutates state; Spectatui is the dashboard and the
  confirmation layer in front of it, exactly like Nx Console never computes
  a dependency graph itself — it calls `nx graph` and renders the result.

This keeps Spectatui safe to update independently of Spec Kit's internal file
formats (which can change between releases) and means every destructive
action goes through the same validation, confirmation prompts, and backup
behavior the CLI already implements (e.g. `extension remove` backs up config
by default unless `--keep-config`/`--force` is passed).

## 1.6. Two different "workflow" concepts in Spec Kit

This needed real CLI exploration to get right, and it directly resolves a
previously-open question (what the status bar's "workflows" stat means).
Spec Kit has **two unrelated things both called "workflow"**:

1. **The lifecycle** — the fixed
   constitution → specify → clarify → plan → tasks → analyze → checklist →
   implement sequence a feature moves through. This doc calls it
   `LifecycleStage` (§4) specifically to avoid the name collision below.
2. **`specify workflow`** — an actual automation/pipeline engine. Verified
   by running `specify init` and inspecting the result: every new project
   ships with one pre-installed workflow,
   `.specify/workflows/speckit/workflow.yml`, registered in
   `.specify/workflows/workflow-registry.json` as:
   ```json
   {
     "schema_version": "1.0",
     "workflows": {
       "speckit": {
         "name": "Full SDD Cycle",
         "version": "1.0.0",
         "description": "Runs specify → plan → tasks → implement with review gates",
         "source": "bundled"
       }
     }
   }
   ```
   i.e. an installable, runnable YAML pipeline that *automates* the
   lifecycle commands end-to-end with review gates — `specify workflow run
   speckit`, with progress trackable via `specify workflow status [run_id]`
   and resumable via `specify workflow resume`. Unlike extensions/presets,
   workflows have **no enable/disable/set-priority/update** subcommands
   (confirmed absent from `specify workflow --help`) — they're run-once
   pipelines, not always-on customizations, so the action model in §5.5 is
   shaped differently on purpose, not by oversight.

**Resolution for the status bar (§6.5)**: "workflows" means installed
automation workflows from `specify workflow list` — exactly one in a
fresh project (the bundled `speckit` one), more if the user installs
others from a workflow catalog. This is a real, CLI-native count, not the
"active features/sessions" placeholder guessed in an earlier draft.

## 2. Feature set (v1)

1. **Feature/session manager** — list every `specs/NNN-name/` feature,
   its current lifecycle stage, and its tmux session status.
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
5. **Integrations manager** — list installed/available coding-agent
   integrations, install/uninstall/switch/use the default one, check
   drift via `integration status --json`, all via `specify integration *`
   (§5.5) — a deliberately different action model from extensions/presets.
6. **Automation workflows manager** — list, run, resume, and check status
   of `specify workflow` pipelines (§1.6, §5.5) — distinct from the
   lifecycle timeline below, which is the fixed stage sequence rather
   than an installable/runnable pipeline.
7. **Lifecycle timeline** — constitution → specify → clarify → plan →
   tasks → analyze → checklist → implement, per feature, inferred from
   which files exist and (optionally) timestamps/git log on them.
8. **Live agent view** — tail of the tmux pane running the agent for the
   selected feature, with a one-key jump to a fully attached session.
9. **Customizable layout** — show/hide panes, rearrange them, switch
   between layout presets; persisted between runs.
10. **Theming** — light / dark / follow-system, persisted.

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
    workflows: Vec<WorkflowInfo>,        // automation workflows, §1.6 — not LifecycleStage
}

struct Feature {
    id: String,                          // e.g. "001-create-taskify"
    branch: Option<String>,              // git branch, if discoverable
    dir: PathBuf,                        // specs/001-create-taskify/
    artifacts: FeatureArtifacts,
    stage: LifecycleStage,                // derived, not stored
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

enum LifecycleStage {
    NotStarted,
    Specified,     // spec.md exists
    Clarified,     // spec.md has a Clarifications section
    Planned,       // plan.md exists
    TasksGenerated,// tasks.md exists
    Analyzed,      // analyze report / marker present
    Implementing,  // tasks.md has some [x] but not all
    Implemented,   // tasks.md fully checked off
}

// Mirrors .specify/extensions/.registry and .specify/presets/.registry,
// which are byte-for-byte the same shape (verified by installing a real
// extension and a real preset and diffing the two registry files).
struct ExtensionInfo {
    id: String,
    version: String,
    source: String,                       // observed: "local" for bundled installs; vocabulary to extend as remote/--from sources are seen
    enabled: bool,
    priority: u8,
    registered_commands: HashMap<String, Vec<String>>, // per-agent, dot-notation ids e.g. "claude" -> ["speckit.specify", ...]
    registered_skills: Vec<String>,        // hyphen-notation, e.g. "speckit-specify" — the actually-invocable names
    installed_at: String,                  // ISO 8601, as stored
    available: bool,                       // true if known only via `search`, not present in the local registry
}

struct PresetInfo {
    id: String,
    version: String,
    source: String,
    enabled: bool,
    priority: u8,
    registered_commands: HashMap<String, Vec<String>>,
    registered_skills: Vec<String>,
    installed_at: String,
    available: bool,
}

// From `specify integration list` (table) + `.specify/integration.json`
// (fast local read for current state) + `specify integration status --json`
// (rich, genuinely machine-readable — file-integrity detail included).
struct IntegrationInfo {
    key: String,                  // e.g. "claude", "copilot", "cursor-agent"
    display_name: String,         // e.g. "Claude Code"
    installed: bool,
    is_default: bool,             // matches .specify/integration.json's default_integration
    cli_required: bool,
    multi_install_safe: bool,
    modified_managed_files: Vec<PathBuf>, // from `integration status --json`, for a health/drift view
}

// From .specify/workflows/workflow-registry.json — see §1.6. Deliberately
// has no `enabled`/`priority` fields: confirmed `specify workflow --help`
// has no enable/disable/set-priority subcommands, unlike extensions/presets.
struct WorkflowInfo {
    id: String,                   // e.g. "speckit"
    name: String,                 // e.g. "Full SDD Cycle"
    version: String,
    description: String,
    source: String,               // observed: "bundled"
    installed_at: String,
    updated_at: String,
}

struct WorkflowRun {
    run_id: String,
    workflow_id: String,
    status: String,               // exact enum values TBD — `workflow run`/`status` weren't exercised end-to-end (§10)
}

struct TmuxSession {
    name: String,
    pane_id: String,
    last_snapshot: String,
    status: SessionStatus,     // Running, Idle, Exited(code)
}

struct StatusBarCounts {
    integrations: u32,   // IntegrationInfo entries with installed == true
    extensions: u32,      // ExtensionInfo entries with available == false (i.e. actually installed)
    workflows: u32,        // WorkflowInfo.len() — resolved meaning, §1.6
    presets: u32,            // PresetInfo entries with available == false
}
```

`LifecycleStage` is derived by checking, in order: does `spec.md` exist? does
it contain a `## Clarifications` section? does `plan.md` exist? does
`tasks.md` exist, and what fraction of its checkboxes are `[x]`? Re-validate
this against whatever the *current* spec-kit template headings are when you
scaffold (templates can change between releases) — treat the exact markdown
structure as a thing to confirm from `.specify/templates/*.md` in the
target project, not hardcode blindly. This is read-only inference only —
it never writes anything back to `specs/`.

`ExtensionInfo`/`PresetInfo` are populated from **two sources**, confirmed
to exist for both (an earlier draft of this doc wasn't sure presets had a
registry file — they do, at `.specify/presets/.registry`, identical shape
to the extensions one): parse the local `.registry` file for a fast,
no-subprocess listing of what's *installed*; call `specify extension list
--available` / `specify preset search` for what's *available but not
installed* (these populate entries with `available: true` and no local
registry data). The registry file is read-only input to Spectatui's
display — never written by Spectatui.

`IntegrationInfo` reads `.specify/integration.json` for instant local
state (which integrations are installed, which is default) and calls
`specify integration status --json` for the richer drift-detection view
(modified/missing managed files) — this is the one place in the whole app
where a `--json` flag is actually available and used directly instead of
parsing table output.

## 5. Extensions/presets manager — CLI action model

This is the Nx-Console-shaped part of Spectatui. Extensions and presets
share one action model — confirmed nearly identical CLI surfaces (§1) and
identical registry file shapes (§4) — while integrations and automation
workflows get their own, genuinely different models in §5.5, since their
real command verbs don't match this shape (no generic add/remove/enable/
disable/set-priority for either).

```rust
enum CliTarget { Extension, Preset }

enum CliAction {
    Search { target: CliTarget, query: Option<String>, tag: Option<String>, author: Option<String> },
    Info { target: CliTarget, id: String },
    Add { target: CliTarget, id: String, priority: Option<u8>, dev_path: Option<PathBuf>, from_url: Option<String> },
    Remove { target: CliTarget, id: String, keep_config: bool, force: bool },
    Enable { target: CliTarget, id: String },
    Disable { target: CliTarget, id: String },
    SetPriority { target: CliTarget, id: String, priority: u8 },
    Update { id: Option<String> },                   // extension-only — presets have no `update` (§1, verified)
    Resolve { name: String },                        // preset-only
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
2. Spectatui builds the `CliAction` and renders the exact resulting command
   line (`specify extension remove my-ext --force`) for the user to review
   — nothing has run yet.
3. For non-destructive actions (`search`, `info`, `list`, `resolve`,
   `catalog list`), run immediately, no confirmation needed.
4. For destructive/mutating actions (`add`, `remove`, `set-priority`,
   `enable`/`disable`, `update`, `catalog add`/`remove`), require an
   explicit confirm keypress before executing. Default to *not* passing
   `--force` so the underlying CLI's own confirmation prompt is the second
   line of defense — only pass `--force` if the user explicitly opts into
   skipping prompts (§10, `ForceMode`).
5. On completion, refresh the relevant `ExtensionInfo`/`PresetInfo` list
   from the registry/CLI so the UI reflects the new state — don't
   optimistically mutate local state.

No JSON output flag exists for `extension`/`preset` `search`/`list`/`info`
(confirmed by running them — plain Rich-formatted tables/text). `version
--features --json` and `integration status --json` (§5.5) are the only two
genuinely machine-readable outputs found anywhere in the CLI. Parse
table-formatted stdout for `search`/`list`/`info` here, expecting this to
need maintenance as CLI output formatting changes across Spec Kit releases.
Treat `.registry` as the more stable source for *currently installed*
state, and reserve text parsing for `search` results that have no local
file equivalent.

## 5.5. Integration & automation-workflow action models

Both verified to have genuinely different command shapes from §5 — not a
gap to fill in, a deliberate distinction.

### Integrations

No add/remove/enable/disable/set-priority. Instead: install-state and
"which one is active" semantics, matching the real subcommands from §1.

```rust
enum IntegrationAction {
    List { catalog: bool },                              // --catalog browses full catalog, not just installed
    Status,                                                // status --json — the rich drift-check view
    Install { key: String, force: bool, options: Option<String> },
    Uninstall { key: String, force: bool },
    Switch { target: String, force: bool, refresh_shared_infra: bool, options: Option<String> },
    Use { key: String, force: bool },                      // change default without uninstalling others
    Upgrade { key: String },
    Search { query: Option<String> },
    Info { key: String },
    CatalogList,
    CatalogAdd { url: String, name: String },
    CatalogRemove { name: String },
    // `scaffold` (creates a new integration package) is a Spec Kit
    // contributor/dev-tooling command, not an end-user TUI action — omitted.
}
```

`Switch` and `Use` both change which integration is "active" but
differently: `use` just changes the default pointer, `switch` actually
uninstalls the previous one's files first. Surface both as distinct
actions in the popup (§6.5) — don't collapse them, the underlying file
operations are meaningfully different and that's exactly the kind of
distinction Nx Console-style confirmation previews exist to make visible.

### Automation workflows

```rust
enum WorkflowAction {
    List,
    Info { workflow_id: String },
    Run { source: String, inputs: Vec<(String, String)> },  // source = installed ID or local YAML path
    Resume { run_id: String },
    Status { run_id: Option<String> },                        // omit run_id to show all runs
    Add { source: String },                                     // catalog ID, URL, or local path
    Remove { workflow_id: String },
    Search { query: Option<String> },
    CatalogList,
    CatalogAdd { url: String, name: String },
    CatalogRemove { name: String },
    // `step` (manage workflow step *types*) is catalog/extension-author
    // tooling, not a day-to-day action — omitted from v1, revisit if
    // step-type browsing turns out to matter for end users.
}
```

`Run` is the one action in this whole app that triggers real, potentially
long-running work (it executes the lifecycle commands end-to-end). Treat
it like the agent-attach flow (§3) rather than a quick CLI call: launch it
as its own tracked job with live status polling via `WorkflowAction::Status
{ run_id }`, not a fire-and-forget `CliJob`. The exact run-status JSON
shape wasn't captured during CLI exploration (no run was actually executed
— it would have meant running the full lifecycle against a throwaway
project) — confirm `workflow status --json`'s shape before building the
run-progress UI (§10).

## 6. Customizable panes

### Model

```rust
enum PaneKind {
    FeatureList,
    SpecBrowser,
    Constitution,
    ExtensionsPresets,
    Integrations,         // §5.5 — install/uninstall/switch/use/upgrade
    Workflows,            // automation workflows (`specify workflow`) — see §1.6, distinct from LifecycleStage
    LifecycleTimeline,
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
  feature list + lifecycle timeline; "Coding" - agent output + spec browser
  side by side; "Audit" - extensions/presets + constitution) selectable with
  a single key, on top of full manual customization.

### Persistence

Store `LayoutConfig`, `ThemeMode` (§7), and `ForceMode` (§10, item 2 —
`NeverForce`/`AlwaysForce` for destructive `specify` actions) in a config
file via the `directories` crate, e.g. `~/.config/spectatui/config.toml`
(XDG on Linux/macOS, `%APPDATA%` on Windows). Project-specific overrides
can live at `.spectatui/config.toml` inside the target repo if the user
wants per-project layouts — this is a Spectatui-only file, not part of
Spec Kit's own override stack, to avoid any confusion with `.specify/`.

## 6.5. Screens

Concrete mockups exist for the screens below (rendered earlier in this
conversation) — this section captures the decisions baked into them so
they survive into implementation.

### Dashboard (default screen)

![Spectatui dashboard mockup](spectatui-dashboard.svg)

Three always-visible regions plus the persistent chrome described below:

- **Feature list** (left sidebar) — one row per `specs/NNN-name/` feature:
  stage badge (abbreviated `LifecycleStage`), feature id, and a session
  status dot (green = tmux session running, gray = no/idle session).
  Selecting a row drives every other pane on screen.
- **Lifecycle stepper** (top right) — the full stage sequence
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
| integrations | The Integration manager from §5.5 — list (with `--catalog` toggle), info, install, uninstall, switch, use, upgrade, search, catalog management. Genuinely different action set from extensions/presets (§5.5) — confirmed by running the real CLI, not assumed. |
| extensions | The full Extensions manager from §5 — search, info, add, remove, enable/disable, set-priority, update, catalog management. |
| workflows | The Automation Workflow manager from §5.5 — list, info, run (with input prompts), resume, status (run history), add, remove, search, catalog management. Resolved meaning per §1.6 — these are `specify workflow` automation pipelines, not active features/sessions. |
| presets | The full Presets manager from §5 — same shape as extensions, minus `update` (presets don't have one, §1) and plus `resolve`. |

**Important architectural point**: popups aren't a separate UI
implementation. The extensions/presets popup renders the exact same
widget function as the optional `ExtensionsPresets` pane from §6, the
integrations popup reuses the `Integrations` pane widget, and the
workflows popup reuses the `Workflows` pane widget — only the framing
differs (overlay box vs. docked pane). This avoids maintaining two copies
of each search/add/remove/confirm flow.

```rust
enum PopupKind { Integrations, Extensions, Presets, Workflows }

struct PopupState {
    kind: PopupKind,
    // Extensions/Presets popups reuse CliJob/CliAction state from §5;
    // Integrations/Workflows popups reuse IntegrationAction/WorkflowAction
    // job state from §5.5 — same Pending/Running/Succeeded/Failed shape,
    // different action enum.
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
  only affects Spectatui's own chrome, not the agent's own TUI when attached.

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
    lifecycle.rs              // LifecycleStage inference (read-only) — was workflow.rs, renamed to avoid clashing with §1.6
    cli.rs                     // SpecifyCliClient: CliAction -> `specify` invocation, streamed output
    registry.rs                  // parse .specify/extensions/.registry (+ presets equivalent)
    extensions.rs                  // ExtensionInfo assembly: registry + CLI search/list
    presets.rs                       // PresetInfo assembly: registry + CLI search/list
    integrations.rs                    // IntegrationInfo via `specify integration list/status` — own action shape, see §5.5
    workflows.rs                         // automation workflows (`specify workflow *`) — see §5.5
    tasks_parser.rs                    // tasks.md -> read-only checklist model
    watch.rs                             // notify -> Event (specs/ + .specify/ for external changes)
  ui/
    mod.rs                   // dispatch active panes per LayoutConfig
    feature_list.rs
    spec_browser.rs
    constitution.rs
    extensions_presets.rs      // list + detail + action menu (search/add/remove/enable/...) — shared by pane and popup
    integrations.rs              // install/uninstall/switch/use/upgrade UI — shared by pane and popup (§5.5)
    popup.rs                   // PopupState overlay rendering (Clear + centered Rect), dispatches to the relevant pane widget fn
    cli_confirm.rs                // command-preview confirmation modal
    cli_output.rs                   // live CliJob output pane
    workflows.rs                      // automation workflow list/run/status — shared by pane and popup (§5.5)
    lifecycle_timeline.rs               // was workflow_timeline.rs, renders LifecycleStage stepper
    agent_output.rs
    statusbar.rs               // persistent chrome: install counts (§6.5) + settings gear, clickable to open popups (§6.5), not a PaneKind
    settings.rs                // gear-icon destination; layout/theme/CLI prefs (content TBD)
    palette.rs                // command palette / layout-preset picker
```

## 9.5. Nx workspace & release tooling

Spectatui will be managed inside an Nx monorepo using the
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

All open questions from earlier drafts are now resolved — items 6, 7, and
9 were specifically resolved by running the real `specify` CLI rather than
reading docs or guessing (see the exploration trail through §1, §1.6, §4,
and §5.5).

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
5. ~~Integrations popup action set~~ — **Has its own action model,
   confirmed genuinely different from extensions/presets, not a guess.**
   Running `specify integration --help` showed install/uninstall/switch/
   upgrade/list/status/use/search/info/scaffold/catalog — no generic
   add/remove/enable/disable/set-priority at all. §5.5 now has the real
   `IntegrationAction` enum built from this, replacing the earlier "mirror
   extensions/presets" assumption entirely.
6. ~~Presets registry file shape~~ — **Confirmed by installing a real
   preset (`lean`) and inspecting the result.** `.specify/presets/.registry`
   exists and is byte-for-byte the same schema as the extensions registry
   (§4). The one-character difference from an earlier draft: the file is
   named `.registry` with **no `.json` extension** despite being JSON —
   fixed throughout this doc.
7. ~~`specify integration list` output shape~~ — **Confirmed by running it
   for real.** Table output for `list`; genuinely structured JSON for
   `integration status --json` specifically (§4, §5.5) — the richest
   machine-readable surface found anywhere in the CLI.
8. ~~`@monodon/rust` generator/executor names~~ — **Still open.** This one
   is outside the `specify` CLI entirely (it's the Nx/Cargo tooling side,
   §9.5) and wasn't part of this round of CLI exploration. Verify against
   the installed plugin version at scaffold time, same caveat as before.
9. ~~What "workflows" means in the status bar~~ — **Resolved with real
   data, not a guess.** It's `specify workflow list`'s count — installed
   automation pipelines (§1.6), exactly one (`speckit`, "Full SDD Cycle")
   in a freshly-initialized project. This *replaces* the earlier "tentative:
   active features/sessions" placeholder in `StatusBarCounts` (§4).

### Still worth checking before/during the relevant build-order step

A short, now much smaller list — most of what would have gone here got
resolved by actually running the CLI (`uvx --from
git+https://github.com/github/spec-kit.git@<tag> specify ...`) rather than
reading docs or guessing:

- [ ] `@monodon/rust` executor/generator names (item 8) — check against
      the installed plugin version when scaffolding (§9.5, build-order
      step 1).
- [ ] Exact `workflow status --json` / `workflow run --json` shape (§5.5)
      — wasn't exercised end-to-end here since it would mean actually
      running the full lifecycle against a throwaway project. Confirm
      before building the run-progress UI (build-order step, automation
      workflow manager).
- [ ] `ExtensionInfo.source`/`PresetInfo.source` vocabulary beyond the one
      observed value (`"local"`, for bundled/default-catalog installs) —
      install something via `--from <url>` to see the remote-source value
      before assuming a fixed enum.



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
10. Lifecycle timeline view, tying `LifecycleStage` history together visually.
