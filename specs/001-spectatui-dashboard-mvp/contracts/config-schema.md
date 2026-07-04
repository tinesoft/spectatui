# Contract: `AppConfig` persisted TOML schema

The one piece of state spectatui itself writes (`crates/spectatui/src/config.rs`),
covering spec FR-026/FR-027/FR-028. This is a contract in the sense that a project's
committed `.spectatui/config.toml` (if any) and a user's own config file must both remain
loadable across spectatui versions within this feature's scope.

## Schema

```toml
theme = "dark"                 # "dark" | "light"
accent = "indigo"              # "indigo" | "teal" | "amber"
dashboard_layout = "overview"  # "overview" | "coding" | "audit"
mouse_support = true           # bool
agent_tail_follow = true       # bool
confirm_before_force = true    # bool
tmux_prefix = "spectatui-"     # string, session-naming prefix

# config_location is NOT itself a persisted field the user sets — it is derived at
# load time from which file path was actually resolved, and surfaced read-only in
# Settings (spec FR-028). Listed here for completeness of the in-memory AppConfig, not
# because it appears as a key a user would hand-edit.

[custom_layout]                # present only if the user has built one; omitted otherwise
# panes = [ { kind = "FeatureList", visible = true, order = 0, size = 2 }, ... ]
```

## Resolution order (read)

1. `<project_root>/.spectatui/config.toml` — project-local override, if present.
2. A fixed, ordered fallback chain of user-level locations (see `research.md` for the
   rationale; the exact paths are an implementation detail, not part of this contract —
   only the precedence rule below is load-bearing).

**Contract invariant**: whichever file is found first wins in full — fields are not
merged across files. A project-local file that omits a field gets that field's built-in
default, not a value inherited from a user-level file.

## Write behavior

- Every settings change made via the Settings screen or a global keybinding
  (theme toggle, accent cycle, layout switch, layout-editor change, mouse-support
  toggle, confirm-before-force toggle, tmux-prefix edit) is written back immediately to
  whichever file was resolved at load time (spec FR-026).
- `--theme`/`--accent` CLI overrides (see `app-cli-args.md`) are **not** written back —
  they affect only the in-memory config for the current run.
- Unknown/future fields in a config file MUST be tolerated (ignored) rather than causing
  a load failure, so older config files remain loadable and newer optional fields can be
  added without a breaking migration.
