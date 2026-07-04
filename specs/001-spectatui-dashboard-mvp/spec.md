# Feature Specification: Spectatui Dashboard — Initial Version

**Feature Branch**: `001-spectatui-dashboard-mvp`

**Created**: 2026-07-04

**Status**: Refined

**Refined**: 2026-07-04 — Documented the in-app coding-agent session launch capability: triggering the start action on a feature with no running session now creates its terminal session (running the project's default coding-agent integration) and hands off to it immediately, instead of requiring the user to manually pre-start a terminal-multiplexer session outside spectatui before attach could find anything.

**Input**: User description: "feature - initial version. from @design/core/spectatui-archi-design.md and current implementation, write me the necessary spec files"

## Clarifications

### Session 2026-07-04

- Q: What's the largest project size (features + installed extensions/presets/integrations/workflows combined) the initial version needs to stay responsive for? → A: Moderate (≤100 total items)
- Q: Should the initial version target compatibility with one pinned range of Spec-Kit template versions, or tolerate drift gracefully? → A: Target current templates only; unrecognized formats degrade to "unknown stage" rather than erroring
- Q: Can more than one CLI-mediated action run at the same time, or must the user wait for the current one to finish? → A: Only one action at a time; starting a new one is blocked/disabled until the current one finishes
- Q: Must session/lifecycle status indicators remain distinguishable without relying on color alone? → A: Color-only is acceptable for v1; each status is already paired with a distinct shape/glyph and text label, so no dedicated colorblind-accessibility work is required

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See every feature's status and live agent activity at a glance (Priority: P1)

A developer using GitHub Spec-Kit on a project opens spectatui from their terminal and immediately sees every feature (`specs/NNN-name/`) in the project, its current place in the spec-driven lifecycle (constitution → specify → clarify → plan → tasks → analyze → implement), and whether a coding agent is actively working on it in a live terminal session — without opening any files or running any commands by hand.

**Why this priority**: This is the core reason spectatui exists — turning scattered file state and background terminal sessions into one glanceable dashboard. Without this, there is no product.

**Independent Test**: Launch spectatui against a project with multiple features in different lifecycle stages, some with active tmux sessions and some without. Verify the feature list, lifecycle stage, and running/idle indicator are all visible without further navigation, and that selecting a different feature updates the lifecycle detail and agent output shown.

**Acceptance Scenarios**:

1. **Given** a project with several `specs/NNN-name/` features, **When** spectatui starts, **Then** every feature appears in the feature list with an abbreviated lifecycle-stage badge and a running/idle indicator for its coding-agent session.
2. **Given** a feature is selected in the feature list, **When** the user views the lifecycle detail, **Then** the full stage sequence is shown with completed stages, the current stage, and (when `tasks.md` exists) a task-completion progress note.
3. **Given** a feature has an active coding-agent tmux session, **When** the user views the agent output area, **Then** the most recent output lines from that session are shown live, updating as the session produces new output.
4. **Given** the user wants to interact with the agent directly, **When** they trigger the attach action, **Then** spectatui hands off the terminal to a full, live, interactive session for that feature, and returns to the dashboard when the user detaches.
5. **Given** a file inside `specs/` or `.specify/` changes on disk (e.g., an agent just wrote `plan.md`), **When** the change is saved, **Then** the dashboard reflects the new lifecycle stage without the user manually refreshing.
6. **Given** a feature has no active coding-agent session, **When** the user triggers the start action on the agent output area, **Then** spectatui creates a new terminal session running the project's default coding-agent integration and immediately hands off to it, the same way the attach action does for a session that was already running.

---

### User Story 2 - Browse a feature's specification artifacts and the project constitution (Priority: P2)

A developer wants to read a feature's `spec.md`, `plan.md`, `tasks.md`, or `research.md`, or the project's `constitution.md`, rendered legibly in the terminal — including task checkboxes and parallelizable-task markers — without leaving spectatui or risking an accidental edit.

**Why this priority**: Reading these artifacts is the second most common action after checking status, and doing it inside the dashboard (versus context-switching to an editor) is a primary value proposition.

**Independent Test**: Select a feature with all artifact types present, open the artifact browser, switch between tabs, and confirm rendering and scrolling work; separately open the constitution viewer from any screen and confirm it always shows the same file regardless of which feature is selected.

**Acceptance Scenarios**:

1. **Given** a feature has `spec.md`, `plan.md`, `tasks.md`, and `research.md`, **When** the user opens its artifact browser, **Then** they can switch between each available document by tab and scroll its rendered content.
2. **Given** `tasks.md` contains checklist items, **When** it is rendered, **Then** each item shows its checked/unchecked state and any parallel-execution marker is visually distinguished from the task description.
3. **Given** an artifact file does not exist for a feature (e.g., no `research.md` yet), **When** the user views that tab, **Then** the tab clearly indicates the document has not been created rather than showing an error or blank confusion.
4. **Given** the user is on any screen, **When** they invoke the constitution shortcut, **Then** the project's `.specify/memory/constitution.md` is shown, independent of which feature (if any) is currently selected.
5. **Given** the user is viewing any artifact, **When** they attempt to modify it, **Then** no edit capability is offered — these views are strictly read-only.

---

### User Story 3 - Manage extensions, presets, integrations, and automation workflows safely (Priority: P3)

A developer wants to see which Spec-Kit extensions, presets, coding-agent integrations, and automation workflows are installed versus available, and to install, remove, enable/disable, reprioritize, switch, or run them — with every change going through the real `specify` CLI and a clear preview/confirmation step, never through spectatui silently rewriting configuration files itself.

**Why this priority**: This is significant, frequently used functionality, but a project can be monitored (P1) and read (P2) without ever managing extensions — so it's correctly ordered after the read-only value.

**Independent Test**: Open each of the four manager views (extensions, presets, integrations, workflows) independently, confirm installed items are listed with their status, trigger a mutating action, confirm the exact command is previewed before anything runs, confirm the action requires explicit confirmation, and confirm the list refreshes from the underlying source afterward.

**Acceptance Scenarios**:

1. **Given** the extensions manager is open, **When** the user views it, **Then** installed extensions show their status (enabled/disabled), priority, and description, sourced from the project's local extension registry.
2. **Given** the user selects a mutating action (e.g., remove an extension, switch the active integration, disable a preset), **When** they trigger it, **Then** spectatui shows the exact resulting command line before anything executes.
3. **Given** a mutating action has been previewed, **When** the user has not yet confirmed it, **Then** nothing changes on disk or in any registry.
4. **Given** the user confirms a mutating action, **When** it completes, **Then** its live output is visible, and the relevant manager list refreshes to reflect the new state without the user manually re-opening it.
5. **Given** the user wants to check an integration's file integrity, **When** they request its status, **Then** any drift from the expected installed files is shown.
6. **Given** the user wants to run an automation workflow, **When** they start or resume one, **Then** its run status is visible without leaving the dashboard, and running/resuming does not require the enable/disable/priority actions that extensions and presets have (workflows are one-off pipelines, not standing customizations).

---

### User Story 4 - Customize and persist the dashboard's layout and appearance (Priority: P4)

A developer wants to arrange which panes are visible and how they're sized, pick one of the built-in dashboard layouts, and choose a color theme and accent — and have those choices remembered the next time they open spectatui on this or another project.

**Why this priority**: Personalization increases day-to-day comfort and efficiency but is not required to get value from the dashboard's core monitoring and browsing capability.

**Independent Test**: Switch between the built-in layouts with a single keypress each; enter the layout editor, hide a pane, reorder two panes, and resize one; toggle theme and accent; restart spectatui and confirm every choice persisted.

**Acceptance Scenarios**:

1. **Given** the dashboard is open, **When** the user presses the layout shortcut keys, **Then** the dashboard switches immediately between the built-in Overview, Coding, and Audit arrangements.
2. **Given** the user enters the custom layout editor, **When** they hide a pane, reorder panes, or resize a pane, **Then** the dashboard's custom layout updates immediately to reflect each change.
3. **Given** the user has toggled the theme or cycled the accent color, **When** any screen is rendered afterward, **Then** the new theme/accent is applied consistently across every screen and popup.
4. **Given** the user has made layout, theme, or accent changes, **When** they quit and relaunch spectatui, **Then** all of those choices are restored exactly as left.
5. **Given** a project provides its own local settings file, **When** spectatui starts inside that project, **Then** the project-local settings take precedence over the user's general settings for this session.

---

### User Story 5 - Navigate and act entirely from the keyboard, with optional mouse support (Priority: P5)

A developer working over SSH or inside tmux (where a mouse may not be usable) wants every action — switching screens, opening a manager, filtering to a specific command, quitting safely — reachable via keyboard shortcuts alone, with mouse interaction available as a convenience when it is usable.

**Why this priority**: Keyboard-first operation is essential to the tool's terminal-native design, but it is an enabling/quality concern layered on top of the functionality delivered by the higher-priority stories rather than a standalone feature.

**Independent Test**: Without touching the mouse, navigate to every screen and every manager popup, filter and execute a command via the command palette, and quit with confirmation; then repeat key actions with mouse clicks where supported and confirm equivalent results.

**Acceptance Scenarios**:

1. **Given** the user is on any screen, **When** they open the command palette and type part of a command's name, **Then** the list filters to matching commands and the selected one can be executed immediately.
2. **Given** the status bar shows counts for integrations, features, extensions, presets, and workflows, **When** the user presses that item's fallback letter key, **Then** the corresponding manager popup opens, identically to clicking it with a mouse.
3. **Given** the user requests to quit, **When** they have not yet confirmed, **Then** spectatui remains running and requires an explicit confirmation keypress before exiting.
4. **Given** a popup is open, **When** the user presses the close key, **Then** the popup closes and the underlying screen's state (selection, scroll position, in-progress actions) is unchanged.
5. **Given** mouse support is enabled in settings, **When** the user clicks a list row, tab, or status-bar item, **Then** the same action occurs as the equivalent keyboard shortcut; **When** mouse support is disabled, **Then** clicks have no effect and all actions remain reachable by keyboard.

### Edge Cases

- What happens when the target project has no `specs/` features yet? The feature list must show an empty, non-error state rather than a blank or confusing screen.
- What happens when the underlying terminal multiplexer is not installed or not running? Session-related indicators must degrade to a clear "unavailable" state rather than crashing or hanging the dashboard.
- What happens when the user triggers the start action for a feature with no running session, but the underlying terminal multiplexer is unavailable or no default coding-agent integration is configured for the project? The start action must not create a broken or partial session or crash the dashboard.
- What happens when the `specify` CLI is not installed or not found on `PATH`? Any action requiring it must fail with a visible, readable error rather than a silent no-op or crash.
- What happens when a triggered CLI action exits with a non-zero status? The failure and its output must be visible to the user, and no list should be optimistically updated as if the action succeeded.
- How does the system handle a project path that doesn't contain a recognizable Spec-Kit structure (no `.specify/`)? It must report this clearly rather than showing empty panels that look like "no data yet."
- What happens when an artifact file (e.g., `plan.md`) is deleted or externally modified while it's currently open in the browser? The view must reconcile with the new on-disk state (including "no longer exists") rather than showing stale content indefinitely.
- What happens when the terminal is resized very small? Panes and popups must remain usable (e.g., via scrolling) rather than panicking or rendering unreadable/overlapping content.
- What happens over an SSH/tmux connection without mouse passthrough? Every mouse-driven action (status bar clicks, pane clicks) must have a working keyboard equivalent.
- What happens when the user tries to start a second CLI-mediated action while one is already running? The new action must be blocked/disabled with a clear indication that one is already in progress, rather than starting a second concurrent process.
- What happens when two destructive actions are queued in quick succession? Each mutating action must be confirmed and completed (or fail) independently — there is no batch/silent auto-confirm path.
- What happens when the coding-agent tmux session for a feature ends while the user is attached to it? Control must return cleanly to the dashboard rather than leaving the terminal in an inconsistent state.
- What happens when a feature's artifacts were produced by an incompatible or unrecognized Spec-Kit template version? The lifecycle stage must show as explicitly "unknown" rather than silently guessing or crashing.

## Requirements *(mandatory)*

### Functional Requirements

**Feature & lifecycle monitoring**

- **FR-001**: System MUST discover every feature under the project's `specs/` directory and list it with a human-readable identifier.
- **FR-002**: System MUST infer and display each feature's current lifecycle stage (not started, specified, clarified, planned, tasks generated, analyzed, implementing, implemented) purely by reading existing files, matched against the current Spec-Kit template conventions — it MUST NOT write to any feature's spec artifacts to determine or record this state. If a feature's artifacts don't match any recognized stage pattern (e.g., produced by an incompatible Spec-Kit template version), system MUST degrade to an explicit "unknown stage" indicator rather than erroring or misreporting a stage.
- **FR-003**: System MUST show, for the selected feature, a task-completion progress indicator when that feature has a `tasks.md` file with checklist items.
- **FR-004**: System MUST indicate, per feature, whether its coding-agent terminal session is currently running or idle, using a distinct shape/glyph in addition to color so the state is not conveyed by color alone.
- **FR-005**: System MUST show a live, continuously updating tail of the selected feature's coding-agent session output.
- **FR-006**: System MUST allow the user to hand off to a full, interactive terminal session for a feature's coding agent, and to return to the dashboard when that session is detached.
- **FR-006a**: System MUST allow the user to create a coding-agent terminal session for a feature that has none running, using the project's default coding-agent integration, and hand off to it immediately upon creation — without requiring the user to manually start a terminal-multiplexer session themselves first.
- **FR-007**: System MUST detect relevant changes to project files on disk (under `specs/` and the Spec-Kit configuration directory) and refresh the affected displayed state without requiring a manual reload.

**Artifact & constitution browsing**

- **FR-008**: System MUST render a feature's specification, plan, tasks, and research documents (when present) as formatted text, including headings, lists, and emphasis.
- **FR-009**: System MUST render `tasks.md` checklist items with their completed/incomplete state visually distinguished, and MUST visually distinguish tasks marked as parallelizable from the rest of the task description.
- **FR-010**: System MUST clearly indicate when a given artifact type does not yet exist for a feature, rather than presenting an empty view indistinguishable from an empty document.
- **FR-011**: System MUST provide access to the project's constitution document from any screen, independent of which feature is currently selected.
- **FR-012**: System MUST treat every feature artifact and the constitution as strictly read-only — no create, edit, or delete action may be offered for them.

**Extensions, presets, integrations, and workflow management**

- **FR-013**: System MUST list a project's installed extensions and presets, each with its enabled/disabled status, priority, and description.
- **FR-014**: System MUST list a project's installed coding-agent integrations, each with its installed state and whether it is the current default.
- **FR-015**: System MUST list a project's installed automation workflows, each with its install state and most recent run summary when available.
- **FR-016**: System MUST allow installing, removing, enabling, disabling, and reprioritizing an extension or preset.
- **FR-017**: System MUST allow installing, uninstalling, switching, and setting the default coding-agent integration, and checking an installed integration for configuration drift.
- **FR-018**: System MUST allow adding, removing, running, resuming, and checking the status of an automation workflow, without offering enable/disable/priority controls for workflows (they are one-off pipelines, not standing customizations).
- **FR-019**: Before executing any action that installs, removes, enables/disables, reprioritizes, switches, or otherwise mutates project or user state, system MUST display the exact underlying command that will be run and require an explicit confirmation from the user.
- **FR-019a**: System MUST allow at most one CLI-mediated action to run at a time; while one is in progress, starting another MUST be blocked until the current one completes.
- **FR-020**: System MUST execute every mutating action by invoking the actual Spec-Kit command-line tool as a subprocess — it MUST NOT directly modify extension, preset, integration, or workflow configuration files itself.
- **FR-021**: System MUST stream a running action's output to the user as it happens and clearly indicate whether the action ultimately succeeded or failed.
- **FR-022**: After a mutating action completes, system MUST refresh the affected list from the underlying source of truth rather than assuming the action's intended result occurred.

**Layout, theming, and settings**

- **FR-023**: System MUST offer a small set of built-in dashboard layout arrangements, each reachable with a single keypress.
- **FR-024**: System MUST allow the user to build a custom layout by showing/hiding, reordering, and resizing the dashboard's panes.
- **FR-025**: System MUST offer at least two color themes and at least three accent color choices, each togglable/cyclable with a dedicated keypress, applied consistently across every screen and popup.
- **FR-026**: System MUST persist the user's layout, theme, accent, and other preference choices between application restarts.
- **FR-027**: System MUST allow a project to override the user's general preferences with a project-local settings file when running inside that project.
- **FR-028**: System MUST provide a settings view where every persisted preference can be reviewed and changed, including a read-only display of which settings file is currently in effect.

**Navigation & interaction**

- **FR-029**: System MUST provide a searchable command list (command palette) that filters as the user types and can execute any listed navigation or management action.
- **FR-030**: System MUST show, at all times, a persistent summary of how many integrations, features, extensions, presets, and automation workflows exist in the current project, each of which opens its corresponding manager when selected.
- **FR-031**: System MUST provide a keyboard-only path to every action also reachable by mouse, so the application is fully usable without mouse input.
- **FR-032**: System MUST require explicit confirmation before quitting the application.
- **FR-033**: System MUST allow the user to accept a project path via a command-line argument at startup, defaulting to the current directory when not specified.
- **FR-034**: System MUST allow the user to override the starting theme and accent via command-line arguments at startup.

### Key Entities

- **Project**: The single Spec-Kit-managed repository spectatui is pointed at; owns the constitution reference and the collected lists of features, extensions, presets, integrations, and workflows. Exactly one Project is active per running instance.
- **Feature**: One `specs/NNN-name/` unit of work; has an identifier, an optional associated git branch, a set of artifacts, and a derived lifecycle stage.
- **Feature Artifacts**: The set of documents that may exist for a feature (specification, plan, tasks, research, and related planning outputs); each is independently optional and read-only.
- **Lifecycle Stage**: The feature's position in the fixed constitution → specify → clarify → plan → tasks → analyze → implement sequence, derived from which artifacts exist and their contents — never stored or written by spectatui.
- **Extension / Preset**: An installable customization of Spec-Kit's commands/templates; has an identifier, version, installed/available status, priority (when installed), and description.
- **Integration**: An installable coding-agent tool binding; has an identifier, display name, installed state, default flag, and whether it requires a separate CLI tool.
- **Automation Workflow**: An installable, runnable pipeline that automates the lifecycle end-to-end; has an identifier, install state, and run-history summary, distinct from the fixed Lifecycle Stage sequence.
- **Coding-Agent Session**: The live terminal session associated with a feature's coding agent; has a running/idle status and recent output used for the live tail and full-attach handoff. Spectatui can create this session itself (running the project's default coding-agent integration) when a selected feature has none, or attach to one already running.
- **User Preferences**: The persisted set of choices (theme, accent, dashboard layout, custom pane arrangement, mouse support, confirmation behavior) that shape the dashboard's appearance and behavior across restarts.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can determine every feature's lifecycle stage and live/idle agent status within 5 seconds of starting the application, without navigating away from the initial screen, for a project with up to 100 total features, extensions, presets, integrations, and workflows combined.
- **SC-002**: A user can read any existing artifact (specification, plan, tasks, research) or the constitution for any feature without ever opening a separate text editor or file browser.
- **SC-003**: 100% of state-changing actions (install, remove, enable/disable, reprioritize, switch, run) require an explicit confirmation showing the exact command beforehand — zero mutating actions occur from a single, un-previewed keypress.
- **SC-004**: A user's chosen layout, theme, and accent are restored correctly in 100% of application restarts.
- **SC-005**: Externally made changes to a feature's files are reflected in the displayed lifecycle stage without any manual refresh action from the user.
- **SC-006**: Every action available via mouse is also completable via keyboard alone, verified across every screen and popup.
- **SC-007**: A user can locate and open any of the five manager views (integrations, features, extensions, presets, workflows) in one keypress or click from any screen.

## Assumptions

- This specification describes the dashboard's initial (v1) capability set as it exists today, spanning both crates (`spectatui-core` engine and `spectatui` UI); it is a monitoring and CLI-mediated management surface, not a replacement for Spec-Kit's own CLI or the coding agent's editing of spec artifacts.
- Extensions and presets are exposed through manager popups and a dashboard pane rather than a dedicated full-screen route in this initial version; introducing a standalone screen for them is a possible future enhancement, not part of this spec.
- Command palette filtering matches on typed text appearing anywhere in a command's name in this initial version; more advanced fuzzy/subsequence matching is a possible future enhancement.
- Catalog search/browse of not-yet-installed extensions, presets, and workflows, and self-check/self-upgrade of the underlying CLI tool, are intentionally out of scope for this initial version — the manager views cover installed-item lifecycle actions (install/remove/enable/disable/priority/switch/run) but not catalog discovery browsing.
- Task-completion progress is tracked as a single overall fraction per feature in this initial version; breaking progress down per user-story phase within `tasks.md` is a possible future enhancement.
- Coding-agent session status in this initial version distinguishes only "running" and "idle" — finer-grained states (e.g., distinguishing a cleanly exited session from one that crashed) are not part of this spec.
- Exactly one project is monitored per running instance; multi-project switching within a single session is out of scope.
- A supporting terminal-multiplexer session host and the Spec-Kit command-line tool are both expected to be present in the user's environment; their absence is handled as a degraded/error state (see Edge Cases) rather than a supported offline mode.
- This initial version targets the current Spec-Kit template conventions rather than maintaining compatibility with multiple historical template versions; artifacts that don't match any recognized pattern degrade to an "unknown stage" indicator instead of being actively parsed by a version-detection system.
- Typical usage is a single project with up to roughly 100 total features, extensions, presets, integrations, and workflows combined; the scrolling list views are not required to remain responsive at significantly larger scale in this initial version.
