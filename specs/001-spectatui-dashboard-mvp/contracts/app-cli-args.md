# Contract: spectatui's own command-line interface

The startup argument contract exposed to the end user (`crates/spectatui/src/main.rs`,
`clap`-derived), covering spec FR-033/FR-034.

| Flag | Short | Type | Default | Effect |
|---|:---:|---|---|---|
| `--project` | `-p` | path | `.` (current directory) | Root of the Spec-Kit project to monitor; passed to `Project::discover()` |
| `--theme` | — | `dark` \| `light` | value from loaded `AppConfig` | Overrides the persisted theme for this run only; does not rewrite the config file |
| `--accent` | — | `indigo` \| `teal` \| `amber` | value from loaded `AppConfig` | Overrides the persisted accent for this run only; does not rewrite the config file |
| `--help` | `-h` | flag | — | Standard clap-generated help |

**Contract invariants**:

1. `--project` MUST accept a relative or absolute path; a non-existent or non-Spec-Kit
   path is not a startup error — it produces the degraded "not a recognized Spec-Kit
   project" state described in `spec.md` Edge Cases, so the user can still open Settings
   or quit.
2. `--theme`/`--accent`, when provided, take effect immediately at startup and are
   layered on top of the loaded `AppConfig` in memory — they are session overrides, not
   persisted preference changes (persisting a theme/accent change only happens through
   the in-app Settings/keybinding toggle, per spec FR-026).
3. No other flags or subcommands exist in v1 (no `spectatui <subcommand>` surface).
