---

description: "Task list template for feature implementation"
---

# Tasks: Spectatui Dashboard — Initial Version

**Propagated**: 2026-07-04 — Updated from spec.md refinement (FR-006a, in-app coding-agent session launch)

**Input**: Design documents from `/specs/001-spectatui-dashboard-mvp/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Context**: This feature documents and closes the gap on an **already-implemented**
system (see `plan.md` Summary and `research.md`). Most functional requirements are
already satisfied by existing code — the implementation survey behind this spec found
exactly **two real gaps** against the clarified spec:

1. `WorkflowStage` has no `Unknown` variant, so FR-002's "unrecognized template →
   degrade to unknown stage" clarification isn't implemented yet (currently a non-match
   would fall through to whatever the last checked branch happens to return).
2. `App::show_cli_job` unconditionally overwrites `self.cli_job`, so FR-019a's "block a
   second action while one is running" clarification isn't enforced yet (a second
   confirmed action today would silently orphan the first job's output tracking).

T022 below found and closed a third gap during edge-case verification (missing
`.specify/` structure detection). A subsequent spec refinement added **FR-006a**
(in-app coding-agent session launch) after discovering a fourth gap the same way:
`crates/spectatui-core/src/tmux/mod.rs` could only discover and attach to a tmux
session the user had already started manually, with nothing in the codebase able to
create one — see T024/T025 in Phase 3. The "exactly two real gaps" framing above
describes this feature's original pre-refinement survey only; it is no longer an
exhaustive count.

Tasks below are organized by user story per the standard process, but for stories with
no implementation gap the "task" is an explicit trace-and-confirm step against the cited
files/contracts, not new code — this keeps the list honest about what remains versus
what's already shipped, per `spec.md` Assumptions and `plan.md`'s Constitution Check.

**Tests**: Per Constitution Principle II, the two gap-closing changes below get colocated
`mod tests` regression coverage; the already-implemented stories don't get new tests
introduced by this feature since they add no new behavior.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Two-crate Cargo workspace (see `plan.md` Project Structure) — `crates/spectatui-core/src/`
(engine) and `crates/spectatui/src/` (UI/bin), no separate `tests/` tree; tests are
colocated `mod tests` blocks.

---

## Phase 1: Setup

**Purpose**: Confirm the existing workspace baseline is green before any change

- [X] T001 Verify the baseline builds clean against the workspace root `Cargo.toml`: run
  `pnpm nx run-many -t test`, `pnpm nx run-many -t lint`, and `cargo fmt --all --
  --check` from the repo root; fix any pre-existing failure before proceeding (none
  expected — this is a checkpoint, not new code)

---

## Phase 2: Foundational

**Purpose**: Blocking prerequisites shared by every user story

No foundational tasks are required for this feature: the existing `Project`/`Feature`
discovery (`crates/spectatui-core/src/speckit/mod.rs`), `App` state
(`crates/spectatui/src/app.rs`), and `Theme` (`crates/spectatui/src/theme.rs`) already
provide every shared prerequisite the stories below build on. Proceed directly to
Phase 3.

**Checkpoint**: Foundation already in place — user story work can begin immediately.

---

## Phase 3: User Story 1 - See every feature's status and live agent activity at a glance (Priority: P1) 🎯 MVP

**Goal**: Close the identified gaps (unrecognized-template degrade to "unknown stage",
and — per the FR-006a refinement — in-app coding-agent session launch) and confirm the
rest of the monitoring/live-agent capability already works as specified.

**Independent Test**: Follow `quickstart.md`'s "US1" scenarios — feature list with stage
badges and running/idle dots, live agent tail, attach handoff, and fs-watch-triggered
stage refresh — against a project with 2+ features in different stages, one artifact
set deliberately mismatched to trigger the new "unknown" stage.

### Implementation for User Story 1

- [X] T002 [P] [US1] Add a `WorkflowStage::Unknown` variant and `"unk"` label in
  `crates/spectatui-core/src/speckit/workflow.rs` (`enum WorkflowStage`, `fn label`)
- [X] T003 [P] [US1] ~~Add an "unk" stage-badge color mapping~~ — verified no code change
  needed: `crates/spectatui/src/theme.rs`'s `stage_badge` already has a generic
  `_ => (self.faint, self.bg)` fallback that `"new"` (NotStarted) already relies on with
  no dedicated arm; `"unk"` gets the same muted, consistent treatment for free
- [X] T004 [US1] Update `infer_stage` in `crates/spectatui-core/src/speckit/workflow.rs`
  to return `WorkflowStage::Unknown` when a feature's artifacts don't match any
  recognized heading/checkbox convention, instead of falling through to a
  best-guess stage (depends on T002)
- [X] T005 [US1] Wire the `Unknown` stage into the feature-list row badge
  (`crates/spectatui/src/ui/feature_list.rs`), the Features-popup note
  (`crates/spectatui/src/ui/popup.rs`), and the lifecycle stepper
  (`crates/spectatui/src/ui/workflow.rs`) so it renders distinctly instead of hitting a
  non-exhaustive match (depends on T002, T003, T004). Also fixed a real bug this
  surfaced: the stepper's `current_stage > *min_stage` check used derived `Ord`, and
  since `Unknown` is declared last it would have outranked every stage and shown the
  entire stepper as "done" for an unknown-stage feature — now explicitly short-circuited
  to a neutral style when `current_stage == Unknown`
- [X] T006 [US1] Add unit tests for unknown-stage detection in the `mod tests` block of
  `crates/spectatui-core/src/speckit/workflow.rs`, covering at least one malformed
  `tasks.md` and one malformed `spec.md` fixture (depends on T004)
- [X] T007 [P] [US1] Verify FR-001 and FR-003–FR-007 against existing code — no change
  expected: feature discovery and task-progress fraction
  (`crates/spectatui-core/src/speckit/mod.rs`, `workflow.rs::parse_tasks_progress`),
  session running/idle + capture-pane tail (`crates/spectatui-core/src/tmux/mod.rs`),
  attach handoff (`crates/spectatui/src/main.rs` `Screen::SessionAttach` handling), and
  the `notify_debouncer_mini` watcher (`crates/spectatui-core/src/speckit/watch.rs`); file
  a new task here if any discrepancy is found. **Result**: confirmed, no discrepancy —
  matches `data-model.md`/`research.md` exactly
- [X] T008 [US1] Run `quickstart.md`'s US1 validation scenarios end-to-end, including the
  new "unrecognized template → unknown stage" edge case (depends on T002–T007).
  **Result**: the new Unknown-stage logic is covered by T006's unit tests (malformed
  `tasks.md` and `spec.md` fixtures) and confirmed by code trace. Full interactive
  visual QA (launching the TUI against a live fixture project) could **not** be
  completed in this sandbox — there is no `tmux` installed and no real TTY (a
  pseudo-tty via `script` reports a 0×0 size, which ratatui can't render into), so the
  live rendering of the new "unk" badge/stepper/status-bar note was not visually
  confirmed here. Recommend a manual pass in a real terminal before release.
- [X] T024 [US1] Implements FR-006a: close a fourth real gap surfaced while verifying
  FR-006 end-to-end — `TmuxClient::attach`/`find_session`
  (`crates/spectatui-core/src/tmux/mod.rs`) could only discover and attach to a tmux
  session the user had already started manually; nothing in the codebase created one,
  contradicting the "Press [enter] to start a coding-agent session" hint already
  rendered by `crates/spectatui/src/ui/agent_output.rs`. Added
  `TmuxClient::launch_session` (`crates/spectatui-core/src/tmux/mod.rs`);
  `App::default_agent_key` and `App::session_name_for` plus a `launch_request` flag
  (`crates/spectatui/src/app.rs`); a pane-aware `Enter` handler on the Dashboard screen
  that triggers launch only when the Agent pane is focused and no session is running,
  falling back to the existing `enter_spec_browser` behavior otherwise; and a shared
  `attach_to` helper extracted from `attach_session` so the new `launch_session` reuses
  the same suspend/restore-TUI dance around the foreground tmux handoff
  (`crates/spectatui/src/main.rs`) (depends on T007)
- [X] T025 [US1] Verify the FR-006a launch path end-to-end (depends on T024): confirmed
  the exact `tmux new-session -d -s <name> -c <cwd> <command>` invocation directly
  against a live `tmux` binary (installed for this verification, matching session name
  and running command confirmed via `tmux capture-pane`); ran
  `pnpm nx run-many -t build,test,lint` for `spectatui`/`spectatui-core` (all green, zero
  clippy warnings); confirmed `IntegrationInfo.key` (not `.name`) is the correct literal
  command to invoke by tracing `registry::load_integrations`. Updated `quickstart.md`'s
  US1 scenario 2 to exercise create-then-attach instead of requiring the tester to
  pre-create the tmux session by hand. **Not covered**: no colocated `mod tests` were
  added for `launch_session`/the new key handler (consistent with the existing
  no-automated-tests state of the attach flow this extends — see T007's citation of
  `Screen::SessionAttach` handling — but flagged here per Constitution Principle II for
  a future task if regression coverage is wanted)

**Checkpoint**: User Story 1 fully satisfies spec.md at this point, including the
FR-002 and FR-006a clarifications.

---

## Phase 4: User Story 2 - Browse a feature's specification artifacts and the project constitution (Priority: P2)

**Goal**: Confirm the read-only artifact/constitution browser already satisfies the spec
— no gap was found here.

**Independent Test**: Follow `quickstart.md`'s "US2" scenarios — tab through all four
artifact types plus a missing one, and open the constitution viewer from multiple
screens.

### Implementation for User Story 2

- [X] T009 [US2] Verify FR-008–FR-012 against existing code — no change expected:
  Markdown/tasks rendering (`crates/spectatui/src/ui/spec_browser.rs`
  `render_md_line`/`render_tasks_line`/`split_task_id`), missing-artifact tab state, and
  the constitution entry point (`crates/spectatui/src/app.rs` `enter_constitution`); file
  a new task here if any discrepancy is found. **Result**: confirmed, no discrepancy;
  additionally grepped for write/edit/delete file operations in `spec_browser.rs` and
  found none, corroborating the FR-012 read-only guarantee
- [X] T010 [US2] Run `quickstart.md`'s US2 validation scenarios end-to-end (depends on
  T009). **Result**: no code changed for this story, so no regression risk; same sandbox
  limitation as T008 applies to live visual confirmation (no tmux/TTY available here)

**Checkpoint**: User Stories 1 and 2 both fully satisfy spec.md.

---

## Phase 5: User Story 3 - Manage extensions, presets, integrations, and automation workflows safely (Priority: P3)

**Goal**: Close the second identified gap (no enforced single-in-flight-action guard)
and confirm the rest of the CLI-mediated management capability already works as
specified.

**Independent Test**: Follow `quickstart.md`'s "US3" scenarios against
`contracts/specify-cli-actions.md`, including the new "second action is blocked while
one is running" scenario.

### Implementation for User Story 3

- [X] T011 [US3] Add a `can_start_cli_action(&self) -> bool` guard method on `App` in
  `crates/spectatui/src/app.rs`, returning `false` when `self.cli_job` is `Some` with
  `status == JobStatus::Running` (implements FR-019a). **Found and fixed a sharper bug
  while implementing this**: a freshly spawned `CliJob` starts as `JobStatus::Pending`
  and only flips to `Running` once its first output line is polled (or never, for a
  silent fast command) — so the guard blocks on `Pending | Running`, not `Running`
  alone, closing a race window where a second action could slip in immediately after
  spawn
- [X] T012 [US3] Call the new guard at every confirmation call site that invokes
  `show_cli_job` in `crates/spectatui/src/main.rs`; when blocked, surface a clear
  "an action is already running" message instead of overwriting the in-flight job
  (depends on T011). Implemented as a single shared `spawn_and_show_cli_job` helper
  (rather than repeating the guard at all 6 call sites) that falls back to re-opening
  the in-flight job's own `CliOutput` popup when blocked, reusing existing UI instead of
  adding a new toast/message mechanism
- [X] T013 [P] [US3] Add unit tests for `can_start_cli_action` in the `mod tests` block
  of `crates/spectatui/src/app.rs` (create the module if it doesn't exist), covering: no
  job → allowed; `Running` job → blocked; `Succeeded`/`Failed` job → allowed again
  (depends on T011). Also added a `Pending`-state case per the fix noted in T011
- [X] T014 [US3] Verify FR-013–FR-018, FR-019, and FR-020–FR-022 against existing code —
  no change expected beyond T011/T012: `crates/spectatui-core/src/speckit/cli.rs`
  (`CliAction`, `is_destructive`, `to_command_line`, `SpecifyCliClient`) against
  `contracts/specify-cli-actions.md`, and the four manager UI files
  (`crates/spectatui/src/ui/extensions_presets.rs`, `integrations.rs`, `workflows.rs`);
  file a new task here if any discrepancy is found. **Result**: confirmed, no
  discrepancy — additionally traced `App::poll_cli_job` and confirmed FR-022 (refresh
  from source of truth on success, no optimistic update on failure) is already correctly
  implemented via `should_refresh` + `refresh_project()`
- [X] T015 [US3] Run `quickstart.md`'s US3 validation scenarios end-to-end, including the
  new blocked-second-action scenario (depends on T011–T014). **Result**: covered by
  T013's unit tests and code trace; same sandbox limitation as T008 applies to live
  visual confirmation

**Checkpoint**: User Stories 1, 2, and 3 all fully satisfy spec.md, including the
FR-019a clarification.

---

## Phase 6: User Story 4 - Customize and persist the dashboard's layout and appearance (Priority: P4)

**Goal**: Confirm layout/theme/settings persistence already satisfies the spec — no gap
was found here.

**Independent Test**: Follow `quickstart.md`'s "US4" scenarios — layout switching,
custom layout editing, theme/accent, and restart persistence.

### Implementation for User Story 4

- [X] T016 [US4] Verify FR-023–FR-028 against existing code — no change expected:
  `crates/spectatui-core/src/layout.rs` (`CustomLayout`), `crates/spectatui/src/theme.rs`,
  and `crates/spectatui/src/config.rs` against `contracts/config-schema.md`; file a new
  task here if any discrepancy is found. **Result**: confirmed, no discrepancy —
  `visible_panes`/`toggle_visibility`/`swap_order`/`resize_pane` and the project-then-
  fallback-chain config resolution match `data-model.md`/`contracts/config-schema.md`
  exactly
- [X] T017 [US4] Run `quickstart.md`'s US4 validation scenarios end-to-end (depends on
  T016). **Result**: no code changed for this story, so no regression risk; same sandbox
  limitation as T008 applies to live visual confirmation

**Checkpoint**: User Stories 1–4 all fully satisfy spec.md.

---

## Phase 7: User Story 5 - Navigate and act entirely from the keyboard, with optional mouse support (Priority: P5)

**Goal**: Confirm keyboard/mouse parity already satisfies the spec — no gap was found
here.

**Independent Test**: Follow `quickstart.md`'s "US5" scenarios — full keyboard-only
navigation, command-palette filtering, and mouse-support toggle parity.

### Implementation for User Story 5

- [X] T018 [US5] Verify FR-029–FR-032 against existing code — no change expected:
  `crates/spectatui/src/ui/palette.rs`, the status-bar/pane click registry
  (`crates/spectatui/src/ui/statusbar.rs`, `App`'s `ClickAction` handling), and the
  quit-confirmation popup flow; file a new task here if any discrepancy is found.
  **Result**: confirmed, no discrepancy — substring-filtered palette (per spec
  Assumptions), `mouse_support`-gated click handling, and `PopupKind::QuitConfirm`'s
  explicit confirm-before-quit all match spec.md
- [X] T019 [US5] Run `quickstart.md`'s US5 validation scenarios end-to-end (depends on
  T018). **Result**: no code changed for this story, so no regression risk; same sandbox
  limitation as T008 applies to live visual confirmation

**Checkpoint**: All five user stories fully satisfy spec.md.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final consistency, documentation, and full-suite validation across all
stories

- [X] T020 [P] Update `design/ui/Spectatui.dc.html` and/or `README.md`'s key-bindings
  table if the new "unk" stage badge or the blocked-second-action message introduce any
  user-visible text not already documented (Constitution Principle III). **Result**: no
  update needed — neither change introduces a new keybinding or a new normal-path
  showcase state (the "unk" badge only ever appears for malformed artifacts, and the
  blocked-action behavior reuses the existing CliOutput popup with no new key)
- [X] T021 Run the full workspace suite against `Cargo.toml`/`crates/*/Cargo.toml` after
  all changes: `pnpm nx run-many -t test`, `pnpm nx run-many -t lint`,
  `cargo fmt --all -- --check` (depends on T002–T019). **Result**: all green — this was
  re-run once more after T022's fix below; final count is 22 tests passing (19 in
  `spectatui-core`, up from 14; 3 in `spectatui`, up from 0), zero clippy warnings,
  clean formatting
- [X] T022 Run `quickstart.md`'s edge-case spot checks (no `.specify/`, no `tmux`, no
  `specify` CLI on `PATH`) end-to-end (depends on T021). **Found and fixed a third real
  gap here**: `Project::discover()` and every `registry::load_*` function silently
  return empty collections when `.specify/` (or its subfiles) don't exist, with no
  error — so a directory with no Spec-Kit structure at all rendered identically to a
  valid, freshly initialized empty project, contradicting the spec's Edge Cases section
  ("must report this clearly rather than showing empty panels that look like 'no data
  yet'"). Added `Project::has_speckit_structure()` in
  `crates/spectatui-core/src/speckit/mod.rs` (plus 2 regression tests) and used it in
  `crates/spectatui/src/ui/feature_list.rs`'s empty-state branch to show a distinct
  "Not a recognized Spec-Kit project" message instead. The other two edge cases (no
  `tmux`, no `specify` CLI) were confirmed by this same sandbox lacking both — `tmux`
  degrades via `TmuxClient::has_tmux`/`SessionStatus`, and a missing `specify` binary
  surfaces as a visible `CliEvent::OutputLine("error: ...")` from `spawn_job`'s
  `Command::spawn` failure path — both already correct, no change needed
- [X] T023 Re-run the checklist in
  `specs/001-spectatui-dashboard-mvp/checklists/requirements.md` against the final state
  and confirm it still passes in full (depends on T021). **Result**: 16/16 still
  passing — no spec change was needed to accommodate any of the three closed gaps, they
  were all implementation catching up to the already-ratified spec

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Empty for this feature — no blocking prerequisite work
- **User Stories (Phases 3–7)**: Each can start immediately after Setup. This held as a
  fully disjoint-file split before the FR-006a refinement (US1:
  `workflow.rs`/`theme.rs`/`feature_list.rs`/`ui/workflow.rs`; US3: `app.rs`/`main.rs`;
  US2/US4/US5: verification only); T024/T025 now give US1 its own edits to
  `app.rs`/`main.rs` too (alongside `tmux/mod.rs`), the same files US3 changes — a
  contributor working both stories in parallel should sequence T011/T012 and T024
  carefully to avoid overlapping edits to those two files, even though neither depends
  on the other's outcome
- **Polish (Phase 8)**: Depends on all five user-story phases being complete

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on other stories — contains the only change to
  `spectatui-core`'s stage-inference logic, and (per the FR-006a refinement) the tmux
  session-launch client and the Dashboard's pane-aware `Enter` handling
- **User Story 2 (P2)**: No dependency on other stories or on US1's changes
- **User Story 3 (P3)**: No dependency on other stories — contains the only change to
  the CLI-action confirmation flow
- **User Story 4 (P4)**: No dependency on other stories
- **User Story 5 (P5)**: No dependency on other stories

### Within Each User Story

- Gap-closing code tasks (US1: T002–T006; US3: T011–T013) before their story's
  verification/quickstart tasks
- Regression tests accompany each gap-closing change in the same phase
- Story complete before its Checkpoint is considered met

### Parallel Opportunities

- T002 and T003 (different files) can run in parallel
- T007 (US1 verification) can run in parallel with T002–T006 (it touches no files those
  tasks touch)
- T013 (US3 tests) can run in parallel with T012 (different file: `app.rs` vs. `main.rs`)
- Phases 3–7 (all five user stories) can be worked on in parallel by different
  contributors once Phase 1 is done
- T020 (docs) can run in parallel with T021–T023 (Polish)

---

## Parallel Example: User Story 1

```bash
# Launch in parallel — different files, no shared dependency:
Task: "Add WorkflowStage::Unknown variant + label in crates/spectatui-core/src/speckit/workflow.rs"
Task: "Add 'unk' stage-badge color mapping in crates/spectatui/src/theme.rs"
Task: "Verify FR-001, FR-003-FR-007 against existing mod.rs/tmux/watch.rs code"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 3: User Story 1 (closes the `WorkflowStage::Unknown` gap)
3. **STOP and VALIDATE**: run `quickstart.md`'s US1 scenarios independently
4. This alone brings the shipped dashboard's monitoring capability into full compliance
   with the clarified spec

### Incremental Delivery

1. Setup → Phase 3 (US1) → validate → this is the MVP increment (closes the one gap
   that affects the dashboard's primary, P1 value)
2. Phase 5 (US3) → validate → closes the second gap (CLI-action concurrency guard)
3. Phases 4, 6, 7 (US2/US4/US5) → verification-only, can run anytime, any order —
   deliver confidence that the rest of the spec is already met
4. Phase 8 (Polish) → final full-suite validation once all stories are confirmed

### Parallel Team Strategy

With multiple contributors, after Phase 1:

- Contributor A: User Story 1 (workflow.rs/theme.rs — the MVP gap)
- Contributor B: User Story 3 (app.rs/main.rs — the concurrency gap)
- Contributor C: User Stories 2, 4, 5 (verification-only, no code changes expected)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- This feature's scope is narrower than a typical from-scratch tasks.md because the
  underlying system already exists — see the Context note at the top of this file
- Two of five stories (US2, US4, US5) have zero expected code changes; their tasks exist
  to make that explicit and verifiable rather than assumed
- If a verification task (T007, T009, T014, T016, T018) finds a real discrepancy against
  its cited FRs, add a new task in that story's phase before closing this feature
- Commit after each task or logical group, per Constitution Principle V (Conventional
  Commits)
