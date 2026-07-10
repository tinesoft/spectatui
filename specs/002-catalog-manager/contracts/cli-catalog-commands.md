# Contract: `specify <kind> catalog *` command surface

This feature's only external interface is the shell-out contract to the
`specify` CLI (there's no HTTP/library API — spectatui is a thin CLI-mediated
dashboard per architecture doc §1.5). This document is the contract between
spectatui's `CliAction` layer and the real `specify` binary: the exact
command lines this feature must produce, and the exact output shapes it must
be able to parse.

## Commands produced (`CliAction::to_command_line()`)

| Action | Command line | Destructive? |
|---|---|---|
| List sources | `specify <kind> catalog list` | No — runs immediately, no confirm |
| Add a source | `specify <kind> catalog add <url> <name> [--priority <N>]` | Yes — preview + confirm required |
| Remove a source | `specify <kind> catalog remove <name>` | Yes — preview + confirm required |

`<kind>` is one of `extension` / `preset` / `integration` / `workflow`
(`CatalogTarget::cli()`). These three command shapes are unchanged from the
existing `CliAction::CatalogList/CatalogAdd/CatalogRemove` implementations in
`cli.rs` — this feature only widens which `<kind>` values they accept (from
2 to 4) and adds callers; it does not change the command-line format itself.

## Output consumed (`list_catalog_sources` / `parse_catalog_urls`)

`specify <kind> catalog list` is run with `COLUMNS=4000` (so the CLI doesn't
line-wrap URLs) and its stdout is parsed as one of two dialects — both
already handled by the existing (soon-to-be-`pub`) `parse_catalog_urls`:

**Dialect A** (priority-annotated):
```text
<name> (priority <N>)
  URL: <url>
  Install: <allowed|discovery only text>
```

**Dialect B** (bulleted, no priority):
```text
- <name> — <install policy text>
  <url>
```
or
```text
[<n>] <name> — <install policy text>
<url>
```

Parsed into:

```rust
pub struct CatalogSource {
    pub name: String,
    pub url: String,
    pub priority: Option<u8>,   // new field — Some(N) for dialect A, None for dialect B
    pub install_allowed: bool,  // false when the policy text contains "discovery only"
}
```

**Failure behavior**: if the process exits non-zero, or the binary isn't on
`PATH`, `list_catalog_sources` returns an empty `Vec` (matching the existing
`catalog_urls` behavior) — the popup shows an empty-state for that kind
rather than an error dialog, consistent with how the app already treats a
missing `specify` CLI elsewhere (`specify_cli_available` gate).

## Consumers

- `crates/spectatui/src/main.rs` — spawns `list_catalog_sources` per kind at
  startup and on manual refresh (`r` key), delivering results via
  `AppEvent::CatalogSourcesLoaded { target, sources }`.
- `crates/spectatui/src/ui/catalogs.rs` — renders whatever `App` currently
  holds for the active `cat_tab`; never calls the CLI directly (UI is a pure
  function of `App` state, per the existing pattern in every other manager
  popup).
- Add/remove go through the existing generic `CliConfirm` → `SpecifyCliClient
  ::spawn_job` → `CliOutput` popup pipeline — no new confirmation/output
  mechanism, this feature only supplies the `CliAction` values.
