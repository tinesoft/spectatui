<!--
Sync Impact Report
==================
Version change: 1.0.0 → 1.0.0 (held at explicit user request — see deviation note)
Rationale: Amendment adds a new Core Principle (V. Conventional Commit Discipline).
Per the Versioning Policy below, adding a principle is normally a MINOR bump
(1.0.0 → 1.1.0). The user explicitly requested the version be left unchanged
for this amendment; the version was NOT bumped, which is itself a deviation
from the stated Versioning Policy. Flagged here rather than silently applied.

Principles established (this amendment):
- V. Conventional Commit Discipline (full @commitlint/config-conventional rule set)

Principles unchanged: I. Code Quality (trimmed duplicate commit-format bullet,
now cross-references Principle V), II. Testing Standards, III. User Experience
Consistency, IV. Performance Requirements.

Added sections: none (new principle only)
Removed sections: none

Templates requiring updates:
- ✅ .specify/templates/plan-template.md — no change needed.
- ✅ .specify/templates/spec-template.md — no change needed.
- ✅ .specify/templates/tasks-template.md — no change needed (unaffected by commit rules).
- ✅ .specify/templates/commands/*.md — no change needed.
- ✅ CONTRIBUTING.md — "Allowed types" table only listed 7 of the 11 types actually
  permitted by `@commitlint/config-conventional`; updated to list the full
  type-enum so it matches what commitlint actually enforces.
- ⚠ README.md — no commit-convention content to update.

Follow-up TODOs:
- TODO(VERSION-POLICY): This amendment was applied without the MINOR bump its
  own Versioning Policy calls for, at explicit user instruction. Next amendment
  should reconcile the version number with the actual principle count (e.g.
  bump to 1.1.0 to reflect five ratified principles) unless deliberately
  deferred again.
-->

# Spectatui Constitution

## Core Principles

### I. Code Quality

Every change MUST compile cleanly and pass static analysis before it is considered
done — this is a Rust workspace (`crates/spectatui-core`, `crates/spectatui`) and
"it compiles" is not the bar.

- `cargo clippy --workspace -- -D warnings` MUST pass with zero warnings. Lint
  suppressions (`#[allow(...)]`) require an inline comment explaining why the
  lint does not apply; blanket suppressions at module or crate level are
  disallowed.
- `cargo fmt --all -- --check` MUST pass; formatting is enforced automatically
  via `lint-staged`/husky on commit and MUST NOT be bypassed with `--no-verify`.
- All commits MUST follow Conventional Commits as enforced by commitlint — see
  Principle V for the full rule set.
- Tasks (build, test, lint, format) MUST be run through Nx (`nx run`,
  `nx run-many`, `nx affected`) rather than invoking `cargo`/`pnpm` tooling
  ad hoc in CI or automation, so caching and affected-graph behavior stay
  correct.
- Prefer explicit error handling (`Result`, `?`) over `.unwrap()`/`.expect()`
  in library and application code paths; `.unwrap()` is acceptable only in
  tests or where a prior check makes failure statically impossible, and such
  cases MUST be justified with a comment.

**Rationale**: The CI pipeline (`ci.yml`) already gates merges on check, test,
clippy, fmt, and commitlint — codifying these as principles means agents and
contributors treat them as non-negotiable design constraints, not late-stage
CI surprises.

### II. Testing Standards

No behavior change ships without a test that would fail without it.

- New logic in `spectatui-core` (parsing, registry/catalog resolution, tmux
  session management, workflow state) MUST include unit tests in a `mod
  tests { ... }` block colocated with the code under test, following the
  existing convention (see `speckit/registry.rs`, `speckit/workflow.rs`).
- Bug fixes MUST include a regression test that reproduces the bug and fails
  on the pre-fix code.
- `cargo test --workspace` (run via `nx run-many -t test`) MUST pass before a
  change is proposed for merge; this is enforced in CI and MUST NOT be
  disabled or skipped for a subset of crates without explicit justification.
- UI rendering logic in `crates/spectatui/src/ui` that has non-trivial branching
  (conditional rendering, layout selection, theme/accent resolution) MUST have
  unit tests validating the branching logic, even where full terminal-buffer
  snapshot testing is impractical.
- Tests MUST be deterministic: no reliance on wall-clock time, real tmux
  sessions, or network access. Fake/mock the `tmux` and filesystem boundaries
  already abstracted in `spectatui-core`.

**Rationale**: This is a terminal UI orchestrating external state (tmux
sessions, Spec-Kit project files) — untested parsing and state-transition code
is the most likely source of silent corruption or a hung TUI, and is the
hardest to debug live in a terminal.

### III. User Experience Consistency

`design/ui/Spectatui.dc.html` is the source of truth for what each screen and
popup renders. UI changes MUST match it, and the mockup MUST be updated first
when a UI change is intentional and not yet reflected there.

- Every interactive screen MUST support both the dark and light theme, and
  all three accent palettes (Indigo, Teal, Amber), with no hardcoded colors
  that bypass the active `Theme`/accent resolution.
- Keybindings MUST be consistent across contexts: global bindings (`t`, `T`,
  `:`/`Ctrl-K`, `?`, `q`, `Ctrl-C`) MUST behave identically from every screen;
  a given local action (navigate, filter, add, remove, enable/disable, close)
  MUST use the same key across popups (Integrations, Extensions, Presets,
  Workflows) unless a documented conflict forces a deviation.
- New popups/screens MUST document their keybindings in `README.md` under
  "Key bindings" in the same table style as existing screens.
- Mouse support, where implemented for a widget class (list rows, tabs,
  status-bar counters, settings chips), MUST behave consistently for every
  instance of that widget class, not just the first one built.
- Error and confirmation states (e.g. quit confirmation) MUST follow the same
  visual and interaction pattern already established, rather than introducing
  a one-off pattern per feature.

**Rationale**: spectatui is a keyboard/mouse-driven dashboard used across many
screens and popups; inconsistency between them (a key that means one thing in
one popup and something else in another, a screen that only renders correctly
in one theme) is directly felt by the user and erodes trust in the tool.

### IV. Performance Requirements

The TUI MUST remain responsive at all times — a terminal dashboard that stutters
or hangs defeats its purpose.

- The main render/input loop MUST NOT block on I/O. Tmux session interaction,
  file reads (spec/plan/tasks/research Markdown, extensions/presets/workflow
  registries), and any subprocess calls MUST happen off the render path
  (async tasks / channels), matching the existing `EventStream` + `tokio`
  interval pattern in `main.rs`.
- Input latency budget: keypresses MUST be reflected in the rendered frame
  within one tick of the input poll interval (currently 100ms); changes that
  would raise this MUST be justified explicitly.
- Background polling/refresh (e.g. the 750ms tmux/session refresh interval)
  MUST be tunable or skippable rather than hardcoded to a lower interval when
  adding new pollers, to avoid compounding CPU/wakeup overhead.
- Rendering MUST avoid unnecessary full-screen redraws or recomputation on
  every tick; expensive derivations (Markdown parsing, catalog parsing) MUST
  be cached and only recomputed when the underlying source actually changes.
- Startup time (cold start to first rendered frame) and release binary size
  are user-facing; the release profile (`lto = true`, `strip = true`,
  `codegen-units = 1`) MUST be preserved, and new dependencies that
  materially regress either MUST be justified in the PR description.

**Rationale**: spectatui runs inside a terminal alongside a live coding agent
session — any perceptible lag or dropped keystroke breaks the "dashboard,
not obstacle" value proposition the project is built on.

### V. Conventional Commit Discipline

Every commit message MUST conform to Conventional Commits as enforced by
`commitlint.config.mjs`, which extends `@commitlint/config-conventional`
with no overrides — the full upstream rule set applies, not a hand-picked
subset.

- Header format MUST be `<type>(<scope>): <subject>`, with `<scope>`
  optional.
- `<type>` MUST be one of the eleven types defined by
  `@commitlint/config-conventional`: `build`, `chore`, `ci`, `docs`, `feat`,
  `fix`, `perf`, `refactor`, `revert`, `style`, `test` — lower-case, never
  empty. Project documentation (e.g. `CONTRIBUTING.md`) MUST list this full
  set, not a narrowed one, so contributors aren't misled into thinking valid
  types (`perf`, `style`, `revert`, `build`) will be rejected.
- `<subject>` MUST NOT be empty and MUST NOT end with a period, per
  `subject-empty`/`subject-full-stop`.
- The commit header MUST NOT exceed 100 characters (`header-max-length`).
- Breaking changes MUST be indicated via a `BREAKING CHANGE:` footer (or `!`
  after the type/scope), never by prose alone in the body — `nx release`
  relies on this to compute MAJOR bumps.
- This rule applies to every commit that reaches `develop`/`main`, including
  squash-merge commit messages; PR titles used as squash messages MUST also
  be Conventional-Commits-compliant.
- The `commit-msg` husky hook MUST NOT be bypassed with `--no-verify`; if a
  hook is bypassed by an unavoidable path (e.g. GitHub UI merge), CI's
  `commitlint` job is the backstop and MUST remain required for merge.

**Rationale**: `nx release` derives version bumps and changelogs purely from
commit messages — a malformed or mistyped commit type doesn't just fail a
lint check, it silently produces a wrong (or missing) version bump at release
time, which is far more expensive to notice and fix after the fact than a
rejected commit.

## Technology & Workspace Constraints

- The project is a Cargo workspace (`crates/spectatui-core`, `crates/spectatui`)
  wrapped by an Nx workspace for task orchestration, caching, and releases.
  Nx MUST remain the entry point for build/test/lint/release tasks; direct
  `cargo`/shell invocations are acceptable for local iteration but MUST NOT
  replace the Nx-orchestrated equivalents in CI or documented workflows.
- `spectatui-core` MUST remain UI-agnostic (no `ratatui`/terminal dependencies)
  so that Spec-Kit domain logic (registry, tmux, workflow) stays independently
  testable and reusable; UI code lives exclusively in `crates/spectatui`.
- Releases are managed by Nx Release from Conventional Commits; version bumps
  MUST NOT be hand-edited in `Cargo.toml`/`package.json` outside of `nx release`.

## Development Workflow & Quality Gates

- All work happens on `feat/<name>` or `fix/<name>` branches off `develop`;
  `main` is release-only and is never committed to directly.
- A change is mergeable only when CI's `Check & Test`, `Format`, and (for PRs)
  `Commit messages` jobs are green — these map directly to Principles I, II,
  and V, and MUST NOT be bypassed.
- PRs that change rendered UI (new/changed screens, popups, themes, keybindings)
  MUST note whether `design/ui/*.dc.html` was updated to match, per Principle III.
- PRs that add a background poller, subprocess call, or expensive computation
  on the render path MUST note the performance impact per Principle IV.

## Governance

This constitution supersedes ad hoc conventions where they conflict. All PRs
and code reviews MUST verify compliance with the five Core Principles above;
a PR description that contradicts a principle without justification MUST be
revised before merge.

**Amendment procedure**: Amendments are proposed via PR modifying this file.
The PR MUST include an updated Sync Impact Report (prepended HTML comment,
as above) describing what changed and why, and MUST identify any dependent
template (`.specify/templates/*.md`) that needs a corresponding update.

**Versioning policy**: This constitution is versioned independently using
semantic versioning:

- **MAJOR** — a principle is removed or redefined in a backward-incompatible
  way (e.g. relaxing a MUST to a SHOULD, or removing a gate).
- **MINOR** — a new principle or materially expanded section is added.
- **PATCH** — wording, clarification, or typo fixes with no rule change.

**Compliance review**: Reviewers are expected to treat unresolved violations
of Principles I–V as blocking, not advisory, unless the PR author has
explicitly called out and justified the deviation in the PR description.

**Version**: 1.0.0 | **Ratified**: 2026-07-04 | **Last Amended**: 2026-07-04
