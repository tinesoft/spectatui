# Implementation Plan: Spectatui Dashboard — Initial Version

**Branch**: `001-spectatui-dashboard-mvp` | **Date**: 2026-07-04 | **Spec**: [spec.md](spec.md)

**Propagated**: 2026-07-04 — Updated from spec.md refinement (FR-006a, in-app coding-agent session launch)

**Propagated**: 2026-07-14 — Updated from spec.md refinement: catalog browsing of not-yet-installed items (previously out of scope) is delivered via the existing inline `/` filter; the direct-catalog-JSON-fetch design choice (already reflected in Technical Context's `reqwest` line above) is now also documented as an intentional exception in `design/core/spectatui-archi-design.md` §1.5, not just an implementation detail.

**Input**: Feature specification from `/specs/001-spectatui-dashboard-mvp/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Spectatui is a tmux-backed, ratatui-rendered terminal dashboard for GitHub Spec-Kit: an
async core engine (`spectatui-core`) discovers `specs/NNN-name/` features and Spec-Kit
registry state, infers lifecycle stage read-only from on-disk artifacts, creates (per
FR-006a) or attaches to a tmux session per feature — running the project's default
coding-agent integration when none exists yet, or handing off to one already running —
and mediates every extension/preset/integration/workflow mutation through the real
`specify` CLI as a confirmed, previewed subprocess; a ratatui/crossterm UI (`spectatui`)
renders that state as a 5-screen, keyboard-and-mouse-driven dashboard with customizable
panes, dark/light theming, and a command palette. This plan documents the
already-implemented v1 architecture (per `design/core/spectatui-archi-design.md` and the
current `crates/spectatui`/`crates/spectatui-core` sources) against the ratified spec and
constitution, so it is traceable and can be extended safely.

## Technical Context

**Language/Version**: Rust, edition 2021, MSRV 1.75 (dev toolchain: 1.96.0)

**Primary Dependencies**: `ratatui` 0.29 (rendering), `crossterm` 0.28 + `event-stream`
(terminal backend/input), `tokio` (full, async runtime), `clap` 4 (CLI args), `directories`
6 (config dir resolution), `serde`/`serde_json`/`serde_yaml`/`toml` (registry & config
parsing), `thiserror` 2 / `anyhow` (errors), `notify` 7 + `notify-debouncer-mini` 0.5
(filesystem watching), `reqwest` 0.12 (rustls-tls, json — read-only direct catalog-JSON
fetch, not used for any mutation), `futures` 0.3 (event/select-loop glue)

**Storage**: No database. Reads Spec-Kit's own files (`.specify/`, `specs/`) read-only;
persists only its own `AppConfig` to a local TOML file (project-local override checked
first, user-level fallback locations otherwise)

**Testing**: `cargo test --workspace` via `nx run-many -t test`; unit tests colocated in
`mod tests { ... }` blocks alongside the code under test (per Constitution Principle II)

**Target Platform**: Cross-platform terminal application — Linux, macOS, Windows
(x86_64 & ARM); runs inside a user's terminal, typically alongside tmux over SSH

**Project Type**: Two-crate Cargo workspace inside an Nx monorepo (`@monodon/rust`) — a
UI-agnostic engine library (`spectatui-core`) and a CLI/TUI binary (`spectatui`) that
depends on it

**Performance Goals**: Keypress-to-frame latency within one input-poll tick (100ms);
tmux session/pane status refreshed on a 750ms interval; filesystem changes reflected
after a 500ms debounce; dashboard remains responsive (SC-001) for a project with up to
~100 total features/extensions/presets/integrations/workflows combined (per spec
Clarifications)

**Constraints**: Render/input loop must never block on subprocess or filesystem I/O
(Constitution Principle IV); at most one CLI-mediated mutating action in flight at a time
(spec FR-019a); release profile (`lto`, `strip`, `codegen-units = 1`) must be preserved;
`spectatui-core` must remain free of UI/terminal dependencies

**Scale/Scope**: Single project per running instance (no project switcher); 5 screens,
4 dashboard layouts, 7 customizable pane kinds, 10 popup kinds, as enumerated in
Project Structure below

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|---|---|---|
| I. Code Quality | `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check` pass; all work lands via Nx-orchestrated tasks and Conventional Commits (see Principle V); no unjustified `.unwrap()`/`.expect()` in library/app code paths | **PASS** — already the workspace's standing practice; this plan introduces no new lint/format exceptions |
| II. Testing Standards | New/changed logic in `spectatui-core` gets colocated `mod tests`; bug fixes get regression tests; `cargo test --workspace` stays green; tests stay deterministic (no real tmux/network) | **PASS** — data-model and contracts below map onto existing, already-tested modules (`speckit::registry`, `speckit::workflow`, `speckit::cli`, `tmux`); no test-hostile design introduced |
| III. User Experience Consistency | `design/ui/Spectatui.dc.html` stays the rendering source of truth; every screen supports both themes and all 3 accents; keybindings stay consistent across popups | **PASS** — this plan documents the existing implementation, which already matches the mockup for the in-scope v1 feature set; known mockup-vs-code deltas (no dedicated Extensions/Presets screen, substring not fuzzy palette filtering) are recorded as explicit spec Assumptions, not silently introduced by this plan |
| IV. Performance Requirements | Render/input loop never blocks on I/O; input latency budget held; background pollers non-blocking and tunable; expensive derivations cached; release profile preserved | **PASS** — matches the existing `EventStream` + `tokio::time::interval` (750ms tmux poll) + `notify_debouncer_mini` (500ms fs debounce) design; no new blocking call introduced |
| V. Conventional Commit Discipline | Every commit for this feature's work follows `@commitlint/config-conventional`'s full type set; breaking changes use `BREAKING CHANGE:`/`!` | **PASS** — process constraint on implementation, not an architectural one; no violation possible at planning time |

No violations identified — the **Complexity Tracking** table below is intentionally empty.

**Post-Phase-1 re-check**: `data-model.md`, `contracts/`, and `quickstart.md` introduce no
new crates, no new blocking I/O on the render/input path, and no new mutation path outside
`SpecifyCliClient` — all five gates above remain **PASS** unchanged.

## Project Structure

### Documentation (this feature)

```text
specs/001-spectatui-dashboard-mvp/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

Existing two-crate Cargo workspace (Nx-managed via `@monodon/rust`); this feature
documents/extends this already-implemented layout rather than introducing a new one:

```text
crates/
  spectatui-core/                 (lib crate — async engine, UI-agnostic)
    src/
      lib.rs
      layout.rs                   # PaneKind / PaneConfig / CustomLayout
      speckit/
        mod.rs                    # Project / Feature discovery (specs/ — read-only)
        workflow.rs                # WorkflowStage inference + tasks progress (read-only)
        cli.rs                     # CliTarget / CliAction / CliJob / SpecifyCliClient
        registry.rs                 # parse .registry + integration.json; fetch available/workflows
        watch.rs                    # notify → filesystem-change events
      tmux/
        mod.rs                     # TmuxClient (list/find/attach/send_keys/launch_session),
                                    # TmuxSession, SessionStatus
  spectatui/                       (bin crate — UI + event loop, depends on spectatui-core)
    src/
      main.rs
      app.rs                       # App state: Screen / DashboardLayout / Pane / PopupKind / SettingsRow
      event.rs                     # Key / Tick / TmuxChanged / FsChanged
      config.rs                    # AppConfig load/save
      theme.rs                     # ThemeMode / Accent / Theme
      ui/
        mod.rs, header.rs, statusbar.rs, feature_list.rs, spec_browser.rs,
        extensions_presets.rs, integrations.rs, workflows.rs, workflow.rs,
        agent_output.rs, session_attach.rs, popup.rs, palette.rs,
        layout_editor.rs, settings.rs

# Tests are colocated `mod tests { ... }` blocks inside the files above
# (Constitution Principle II), not a separate tests/ directory.
```

**Structure Decision**: Single two-crate workspace (engine + bin), matching
`design/core/spectatui-archi-design.md` §9 exactly. No new crates, directories, or a
`tests/` tree are introduced by this plan — it documents the structure already in place
so future feature work in this project has a traceable baseline to build from.

## Complexity Tracking

No Constitution Check violations were identified — this section is intentionally empty.
