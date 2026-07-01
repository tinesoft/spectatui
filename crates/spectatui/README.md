# spectatui

A terminal UI dashboard for [GitHub Spec-Kit](https://github.com/tinesoft/speckit) — track features, manage specifications, and monitor AI agent workflows, all from your terminal.

## Installation

```sh
cargo install spectatui
```

Or download a pre-built binary from the [Releases page](https://github.com/tinesoft/spectatui/releases).

## Usage

```sh
spectatui [OPTIONS]

Options:
  -p, --project <PATH>   Path to the Spec-Kit project root [default: .]
      --theme <THEME>    Override theme: dark or light
      --accent <ACCENT>  Override accent: indigo, teal, or amber
  -h, --help             Print help
```

## Features

- Multi-pane dashboard with Overview, Coding, Audit, and Custom layouts
- Spec / Plan / Tasks / Research browser with rendered Markdown
- Workflow stepper with visual stage tracking
- Extensions, Presets, Integrations, and Workflows managers
- Live agent output pane and session attach
- Command palette, settings editor, dark/light themes

## License

[MIT](https://mit-license.org/) © 2026 Tine Kondo
