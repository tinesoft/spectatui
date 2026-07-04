# Phase 0 Research: Spectatui Dashboard — Initial Version

This feature documents an already-implemented system rather than greenfield work, so
Technical Context in `plan.md` was filled directly from `Cargo.toml`/`Cargo.lock` and a
source-level survey — no `NEEDS CLARIFICATION` markers remain. This document records the
rationale behind the non-obvious technology and pattern choices already embodied in the
code, for traceability, plus the alternatives that were implicitly rejected.

## Decision: ratatui + crossterm for rendering/input

- **Decision**: `ratatui` 0.29 for immediate-mode terminal rendering, `crossterm` 0.28
  (with `event-stream`) for the terminal backend, raw mode, and mouse input.
- **Rationale**: Both are the de facto standard for Rust TUI applications with active
  maintenance, cross-platform (Linux/macOS/Windows) support, and first-class async event
  streaming (`crossterm::event::EventStream`) that composes with `tokio::select!` for the
  main loop — required to interleave keyboard/mouse events, a render tick, tmux polling,
  and filesystem-watch events without blocking any of them on the others.
- **Alternatives considered**: `cursive` (higher-level widget framework, less control over
  custom pane layout and popup overlay compositing needed here); raw `termion`/manual ANSI
  (no Windows support, no mouse abstraction, would reinvent what crossterm already solves).

## Decision: tokio async runtime, subprocess-based tmux and `specify` CLI control

- **Decision**: `tokio` (full features in the bin crate; `process`/`sync`/`rt`/`io-util`/
  `macros` in the core crate) drives everything — the event loop, tmux `capture-pane`/
  `send-keys`/`attach` via `tokio::process::Command`, and every `specify` CLI invocation
  via the same mechanism, streaming stdout/stderr line-by-line as it arrives.
- **Rationale**: Spectatui deliberately never re-implements Spec-Kit's install/removal/
  priority logic or drives tmux through a native library — every mutation is a real
  subprocess whose exact command line is shown to the user before running (spec FR-019,
  FR-020). This keeps spectatui safe to update independently of Spec-Kit's internal file
  formats and gives the user the same confirmation/backup behavior the CLI already
  implements.
- **Alternatives considered**: A tmux control-mode (`-CC`) persistent connection (more
  efficient for very high-frequency polling, but adds protocol-parsing complexity for a
  poll interval — 750ms — that doesn't need it); a native Spec-Kit Rust library
  dependency instead of shelling out (would need to track Spec-Kit's internal format
  changes directly rather than being insulated by the CLI boundary — rejected per the
  Nx-Console-style architecture principle in `design/core/spectatui-archi-design.md` §1.5).

## Decision: single in-flight `CliJob`, no action queue

- **Decision**: `App` holds `cli_job: Option<CliJob>` — at most one mutating CLI action
  runs at a time; starting a new one while one is in flight is blocked (spec FR-019a,
  resolved via `/speckit-clarify`).
- **Rationale**: Confirmed as the deliberate v1 scope during clarification — it keeps the
  output log unambiguous (one command's output at a time) and avoids a queue/concurrent-
  job UI that the spec doesn't require for this version.
- **Alternatives considered**: A job queue with sequential auto-run, or true concurrent
  jobs with multiplexed output — both rejected for v1 as unnecessary scope for a
  single-operator local tool; noted as a possible future enhancement if a real need for
  concurrent installs/switches emerges.

## Decision: hybrid refresh — `notify` file watching + fixed-interval tmux polling

- **Decision**: `notify` + `notify-debouncer-mini` (500ms debounce) watch `specs/` and
  `.specify/` and trigger an event-driven `Project::discover()` re-run; a separate
  `tokio::time::interval` (750ms) polls tmux session/pane state, since tmux has no
  filesystem-change-style notification mechanism.
- **Rationale**: Filesystem changes (an agent writing `plan.md`) are inherently
  event-driven and infrequent, so watching is efficient; tmux status/output has no
  equivalent push mechanism, so polling is the only option, and 750ms balances
  responsiveness (spec SC-005) against CPU/wakeup overhead (Constitution Principle IV).
- **Alternatives considered**: Polling both at a fast fixed interval (simpler, but wastes
  CPU on idle projects and is exactly what Constitution Principle IV calls out to avoid);
  a single unified poll loop for both concerns (rejected — conflates two very different
  change frequencies and would force the faster of the two intervals onto both).

## Decision: hand-rolled Markdown/tasks renderer, no `pulldown-cmark`

- **Decision**: A line-based renderer (`ui/spec_browser.rs`) handles the specific,
  constrained Markdown subset Spec-Kit templates actually produce (headers, bullets,
  fenced code as styled text, bold, rules) plus a dedicated `tasks.md` checklist parser
  recognizing `- [x]`/`- [ ]` and `[P]` parallel markers.
- **Rationale**: Spec-Kit's generated documents use a small, predictable subset of
  Markdown; a full CommonMark parser is unnecessary weight for read-only terminal
  rendering, and a hand-rolled line scan is trivial to keep in lockstep with the specific
  conventions `.specify/templates/*.md` actually uses (per spec Clarifications: target
  current templates only, degrade to "unknown" rather than erroring on drift).
- **Alternatives considered**: `pulldown-cmark` (full CommonMark compliance is unneeded
  surface area and would still require a custom ratatui-widget renderer on top of its
  AST — no net simplification for this constrained use case).

## Decision: direct catalog-JSON fetch (`reqwest`) for available-item listings, read-only

- **Decision**: `registry.rs` fetches extension/preset catalog JSON directly via
  `reqwest` from catalog source URLs (themselves discovered via `specify extension/preset
  catalog list` output), rather than shelling out to `specify extension list --available`
  for every catalog on every refresh.
- **Rationale**: This is a read-only, non-mutating optimization — it never writes
  anything and never substitutes for the CLI on any install/remove/enable/disable/
  priority/switch/run action (those all still go through `SpecifyCliClient`), so it does
  not violate the "CLI mediates all mutation" architecture principle; it only avoids
  redundant subprocess spawns for a purely informational listing.
- **Alternatives considered**: Always shelling out to `specify` for catalog listings
  (simpler mental model, but noticeably slower when multiple catalogs are configured);
  caching CLI output instead of fetching JSON directly (would still incur the initial
  subprocess cost and Rich-table-parsing fragility noted in the architecture doc §5).

## Decision: config precedence — project-local override, then a fixed fallback chain

- **Decision**: `AppConfig` checks a project-local `.spectatui/config.toml` first; if
  absent, it falls back through a fixed, ordered list of user-level locations.
- **Rationale**: Confirmed by source inspection (`config.rs`) as the actual, working
  behavior; a project can pin team-shared preferences (spec FR-027) while individual
  users still get a working default without one.
- **Alternatives considered**: XDG-only resolution via the `directories` crate with no
  project override (simpler, but would drop the team-shared-preferences capability the
  spec requires); documented in spec Assumptions that the exact fallback chain is an
  implementation detail not fixed by the spec itself — only the precedence rule
  (project overrides user-level) is a hard requirement.
