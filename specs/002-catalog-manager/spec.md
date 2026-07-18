# Feature Specification: Catalog Manager

**Feature Branch**: `002-catalog-manager`

**Created**: 2026-07-10

**Status**: Refined

**Refined**: 2026-07-11 — Documented three add-source-form behaviors that
landed during implementation but were missing from the spec: starting the
add flow while a discovery-only source is selected now pre-fills the form
from that source (streamlining the remove-and-re-add pattern used to
install or reprioritize it); the form supports standard text-editing
affordances (cursor movement, forward/backward delete, mouse click-to-position,
paste); and Ctrl+C is scoped to clear the form's input instead of triggering
the app's global quit while the form is open.

**Refined**: 2026-07-14 — Delivered reprioritize/toggle-in-place for extension
and preset catalog sources (previously out of scope): an `e` edit action
sequences a real remove-then-re-add behind one confirm step, rather than
requiring the user to do both steps manually. Integration/workflow catalog
sources remain remove/re-add only, since the underlying tool gives those
kinds no priority or install-allowed concept to edit in the first place.

**Input**: User description: "Catalog Manager: a unified popup in spectatui for managing catalog *sources* across all four Spec Kit resource kinds — extensions, presets, integrations, and workflows. Each of these resource kinds resolves its installable items from one or more catalog sources (a name + URL + priority + install-allowed/discovery-only flag), and today spectatui has no way to view or manage those sources at all, even though it already has full managers for the resources themselves (Extensions, Presets, Integrations, Workflows popups). The Catalog Manager popup should be reachable from a status bar stat, a global keypress, and a command-palette entry; be tabbed by resource kind; support add/remove/refresh per source, each delegated to the underlying catalog tool with a preview-then-confirm flow; and reuse the existing generic confirm/output flow already used by the other managers. Since the global key is being claimed by Catalogs, the existing Constitution viewer keybinding moves to a different key to free it up. Out of scope: reprioritizing or toggling a source in place — done via remove + re-add instead."

## Clarifications

### Session 2026-07-10

- Q: When the Catalog Manager is opened, which resource-kind tab is shown
  initially? → A: Remembers the last-viewed resource kind across opens when
  reached via the status-bar indicator or the global keypress; the
  command-palette entry always resets to Extensions as a predictable,
  discoverable default.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a new catalog source (Priority: P1)

A user wants to start discovering extensions, presets, integrations, or
workflows from a catalog source that isn't configured yet (for example, a
community catalog alongside the official one). They open the Catalog Manager,
pick the resource kind, and add the new source by providing its location, a
name, and an optional priority — without needing to know or type the
underlying command-line syntax themselves.

**Why this priority**: Without the ability to add a source, the feature has
no value — users would still need to fall back to a terminal, defeating the
purpose of a dashboard.

**Independent Test**: Can be fully tested by opening the Catalog Manager,
adding a source for one resource kind, and confirming it appears in that
kind's list afterward — delivers value on its own even if remove/refresh
aren't used in the same session.

**Acceptance Scenarios**:

1. **Given** the Catalog Manager is open on a resource kind's tab showing its
   currently configured sources, **When** the user starts adding a source and
   supplies a location, a name, and (optionally) a priority, **Then** the
   system shows the exact action it is about to take and requires explicit
   confirmation before doing anything.
2. **Given** the user has confirmed adding a source and the underlying action
   succeeds, **When** the action completes, **Then** the new source appears
   in that resource kind's list.
3. **Given** the user has confirmed adding a source and the underlying action
   fails (e.g. unreachable location, invalid input), **When** the action
   completes, **Then** the user sees the failure detail and the existing list
   of sources is left unchanged.
4. **Given** a discovery-only source (not install-allowed) is selected under
   a resource kind's tab, **When** the user starts adding a source, **Then**
   the add form is pre-filled with that source's location, name, and
   priority (if set), so converting it to install-allowed or adjusting its
   priority via remove-and-re-add doesn't require retyping the entry from
   scratch.

---

### User Story 2 - Remove an unwanted catalog source (Priority: P1)

A user no longer trusts or needs a catalog source (for example, it was added
for a one-off evaluation, or it's no longer maintained). They open the
Catalog Manager, select that source under its resource kind, and remove it.

**Why this priority**: Managing sources is incomplete without the ability to
undo an addition or retire a stale one — this is the other half of the core
value alongside adding sources.

**Independent Test**: Can be fully tested by selecting an existing catalog
source and removing it, then confirming it no longer appears in that
resource kind's list — independently valuable and testable without touching
add or refresh.

**Acceptance Scenarios**:

1. **Given** a resource kind's tab with at least one configured source,
   **When** the user selects a source and chooses to remove it, **Then** the
   system shows the exact action it is about to take and requires explicit
   confirmation before doing anything.
2. **Given** the user has confirmed removal and the underlying action
   succeeds, **When** the action completes, **Then** the source no longer
   appears in that resource kind's list.
3. **Given** the user has confirmed removal and the underlying action fails,
   **When** the action completes, **Then** the user sees the failure detail
   and the source remains in the list, since it was not actually removed.

---

### User Story 3 - Browse across all four resource kinds and refresh (Priority: P2)

A user manages sources for more than one resource kind (say, extensions and
workflows) and wants to review all of them without hunting through separate
screens, and to confirm the displayed list matches reality if they suspect it
changed outside spectatui.

**Why this priority**: This is what makes the manager "unified" rather than
four separate one-off tools — valuable, but the feature is still usable
without it if a user only ever manages one resource kind's sources.

**Independent Test**: Can be fully tested by opening the Catalog Manager,
switching between all four resource kinds, and triggering a refresh on one of
them — delivers value even without ever adding or removing a source in the
same session.

**Acceptance Scenarios**:

1. **Given** the Catalog Manager is open, **When** the user switches to a
   different resource kind, **Then** they see that kind's sources without
   closing and reopening the manager.
2. **Given** the Catalog Manager is open on a resource kind's tab, **When**
   the user triggers a refresh, **Then** the displayed list is re-fetched and
   updated to reflect the current state.
3. **Given** the user last viewed the Workflows tab and closed the Catalog
   Manager, **When** they reopen it via the status-bar indicator or the
   global keypress, **Then** it reopens on the Workflows tab; **When** they
   instead open it via the searchable command list, **Then** it opens on the
   Extensions tab regardless of what was last viewed.

---

### Edge Cases

- What happens when a resource kind has zero configured catalog sources?
  The manager shows an empty state for that kind's tab and still allows
  adding a source.
- How does the system handle a failed add/remove (unreachable location,
  invalid name, or the underlying catalog tool being unavailable)? The
  failure detail is surfaced to the user, and the displayed list is not
  optimistically changed — it only reflects what the underlying tool
  confirms actually happened.
- What happens if the user tries to remove the only remaining source for a
  resource kind? The removal is still permitted; the consequences of a
  resource kind having no sources are owned by the underlying system, not by
  this feature.
- What happens if the user opens the Catalog Manager while an unrelated
  action elsewhere in the app is still in progress? Opening the manager does
  not interrupt or need to wait on unrelated in-flight actions.
- How does a user correct a mistake while entering a source's location,
  name, or priority? The add form supports standard text-editing
  affordances — moving the cursor left/right/home/end, forward and
  backward delete, positioning the cursor with a mouse click, and pasting
  text — so a mistake doesn't require retyping the whole entry.
- What happens if the user presses Ctrl+C while the add-source form is
  open? Ctrl+C is scoped to clear the form's input (so the user can start
  over) instead of triggering the app's global quit-on-Ctrl+C behavior;
  Ctrl+C still quits the app everywhere else.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a single view where a user can see
  catalog sources for all four resource kinds (extensions, presets,
  integrations, workflows).
- **FR-002**: Users MUST be able to switch between the four resource kinds
  within that view without leaving it.
- **FR-003**: For the selected resource kind, the system MUST display each
  configured catalog source's name, location, priority (if set), and whether
  it is available for installing items or discovery only.
- **FR-004**: Users MUST be able to add a new catalog source for the selected
  resource kind by providing its location, a name, and optionally a priority.
- **FR-005**: Before adding a source, the system MUST show the user exactly
  what action will be taken and require explicit confirmation.
- **FR-006**: Users MUST be able to remove an existing catalog source for the
  selected resource kind.
- **FR-007**: Before removing a source, the system MUST show the user exactly
  what action will be taken and require explicit confirmation.
- **FR-008**: Users MUST be able to manually refresh the displayed list of
  sources for the selected resource kind so it reflects the current state.
- **FR-009**: The system MUST report the outcome (success or failure) of
  every add/remove action, including failure detail, to the user.
- **FR-010**: The Catalog Manager MUST be reachable through the same three
  kinds of entry points already used for the app's other resource managers
  (extensions, presets, integrations, workflows): a persistent summary
  indicator, a single keypress, and a searchable command list.
- **FR-011**: The persistent summary indicator MUST reflect the total number
  of configured catalog sources across all four resource kinds.
- **FR-012**: The existing Constitution viewer MUST remain reachable by a
  single keypress after the Catalog Manager claims the keypress currently
  used for it; the two features MUST NOT share the same key.
- **FR-013**: The system MUST NOT alter catalog configuration directly —
  every add/remove MUST be delegated to the existing underlying catalog
  management tool, consistent with how the app's other resource managers
  already operate (view/confirm/delegate, never edit local state directly).
- **FR-014**: When opened via the persistent summary indicator or the global
  keypress, the Catalog Manager MUST show whichever resource kind's tab was
  last viewed (sticky across opens); when opened via the searchable command
  list, it MUST always reset to the Extensions tab.
- **FR-015**: When the user starts adding a source while a discovery-only
  source is selected, the system MUST pre-fill the add form with that
  source's location, name, and priority (if set); the form MUST start empty
  otherwise (nothing selected, an empty list, or an install-allowed source
  selected).
- **FR-016**: The add-source form MUST support standard text-editing
  affordances — cursor movement (left/right/home/end), forward and
  backward delete, mouse click-to-position, and paste — so users can
  correct input without retyping the whole entry.
- **FR-017**: While the add-source form is open, Ctrl+C MUST clear the
  form's input rather than triggering the app's global quit action; Ctrl+C
  MUST retain its global quit behavior everywhere else in the app.

### Key Entities

- **Catalog Source**: A named location that a resource kind (extension,
  preset, integration, or workflow) can pull installable items from. Key
  attributes: name, location, priority (optional, affects resolution order
  when multiple sources apply), and whether it's allowed to be installed
  from directly or is discovery-only.
- **Resource Kind**: One of extensions, presets, integrations, or workflows —
  the four groupings whose catalog sources this feature manages.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can reach the list of configured catalog sources for any
  of the four resource kinds in two actions or fewer from anywhere in the
  app.
- **SC-002**: A user can add a new catalog source and see it reflected in the
  list after exactly one confirmation step.
- **SC-003**: A user can remove an unwanted catalog source after exactly one
  confirmation step.
- **SC-004**: 100% of add/remove actions show the user the exact action about
  to be taken before it takes effect — no action mutates catalog state
  silently.
- **SC-005**: Existing users of the Constitution viewer retain single-keypress
  access to it after this feature ships — no existing capability is lost.

## Assumptions

- The existing underlying catalog management tool (already used by the app's
  other resource managers) is the sole source of truth for catalog source
  state; this feature is a visualization and control layer on top of it, not
  an independent store of its own.
- ~~Reprioritizing an existing catalog source is accomplished by removing it
  and re-adding it with a new priority... something the user toggles in
  place~~ — **delivered** (see 2026-07-14 refinement note): an `e` "edit"
  action, scoped to extension/preset catalog sources only (the underlying
  tool's integration/workflow catalog sources have no priority or
  install-allowed concept to edit), opens a form pre-filled with the
  source's current url/name/priority/install-allowed and, on submit, still
  performs a real remove-then-re-add — the underlying tool has no dedicated
  reorder/edit verb today — but as one sequenced pair of calls behind a
  single confirm step instead of the user doing it manually in two.
- Exactly four resource kinds exist today (extensions, presets, integrations,
  workflows); no other kinds are in scope.
- Reachability for this feature reuses the same three entry-point mechanisms
  (status indicator, single keypress, command list) the app's other resource
  managers already use, rather than introducing a new discovery pattern.
