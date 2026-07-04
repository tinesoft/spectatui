# Contributing to spectatui

Thank you for your interest in contributing!

---

## Development setup

spectatui uses a **devcontainer** to ensure a consistent, reproducible environment.

### Prerequisites

- [Docker](https://www.docker.com/)
- [VS Code](https://code.visualstudio.com/) with the [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension

### Getting started

1. Clone the repository:
   ```sh
   git clone https://github.com/tinesoft/spectatui.git
   cd spectatui
   ```
2. Open in VS Code and click **Reopen in Container** when prompted (or run `Dev Containers: Reopen in Container` from the command palette).
3. The container post-create script (`/.devcontainer/post-create.sh`) runs automatically and:
   - Installs pnpm dependencies (`pnpm install`)
   - Sets up shell aliases (`ll`, `cl`, `cc`, `cr`, `cch`)
4. The container includes **Rust 1.96.0** and all required tooling.

---

## Git workflow

This project follows a **develop-based git flow**:

| Branch | Purpose |
|--------|---------|
| `main` | Release-ready code only. Never commit directly. |
| `develop` | Integration branch. All feature branches merge here. |
| `feat/<name>` | New features, branched from `develop`. |
| `fix/<name>` | Bug fixes, branched from `develop`. |

### Typical workflow

```sh
git checkout develop
git pull
git checkout -b feat/my-feature

# ... make changes ...

git push -u origin feat/my-feature
# Open a PR targeting develop
```

Releases are cut by merging `develop` → `main` and running `pnpm release`.

---

## Commit conventions

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/) — this is enforced automatically by [commitlint](https://commitlint.js.org/) via a git hook (husky).

### Format

```
<type>(<scope>): <short description>
```

### Allowed types

| Type | When to use |
|------|-------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `chore` | Build process, dependencies, tooling |
| `docs` | Documentation only changes |
| `refactor` | Code restructuring without behavior change |
| `test` | Adding or fixing tests |
| `ci` | CI/CD configuration changes |
| `perf` | A performance improvement |
| `style` | Formatting/whitespace changes with no code meaning change |
| `build` | Changes to the build system or external dependencies |
| `revert` | Reverts a previous commit |

### Examples

```
feat(ui): add command palette keyboard shortcut
fix(tmux): handle session detach edge case
chore(deps): update ratatui to 0.30
```

---

## Building

```sh
# Via Nx (recommended)
pnpm nx build spectatui

# Via cargo directly
cargo build -p spectatui

# Release build
cargo build --release -p spectatui
```

---

## Testing

```sh
# All crates via Nx
pnpm nx run-many -t test

# Via cargo
cargo test --workspace
```

---

## Linting

```sh
# Via Nx
pnpm nx run-many -t lint

# Via cargo directly
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --all -- --check
```

---

## Release process

Releases are managed by [Nx Release](https://nx.dev/features/manage-releases) with conventional commits:

```sh
# Preview what would change
pnpm release:dry

# First release
pnpm release:first

# Subsequent releases
pnpm release
git push && git push --tags
```

Pushing the tag triggers the GitHub Actions release workflow, which cross-compiles binaries for all platforms and attaches them to the GitHub Release.
