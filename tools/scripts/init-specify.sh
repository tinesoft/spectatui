#!/usr/bin/env bash
# Install the Specify CLI (Spec Kit) at a pinned release, via `uv tool install`.
#
# spectatui shells out to `specify` at startup to discover catalog sources for
# the extensions/presets/integrations/workflows popups, so the CLI must be on
# PATH in the dev environment. Called from .devcontainer/post-create.sh; safe
# to re-run (`--force` reinstalls/upgrades in place).
#
# Requires: uv (https://docs.astral.sh/uv/). The `specify` shim is installed
# into ~/.local/bin, which the devcontainer already has on PATH.

set -euo pipefail

SPECIFY_VERSION="v0.12.4"

if ! command -v uv >/dev/null 2>&1; then
    echo "error: 'uv' is required but not found on PATH — see https://docs.astral.sh/uv/" >&2
    exit 1
fi

echo "Installing Specify CLI ${SPECIFY_VERSION}..."
uv tool install specify-cli --force \
    --from "git+https://github.com/github/spec-kit.git@${SPECIFY_VERSION}"

specify init --here --force --script=sh --integration=claude --integration-options=""

specify --version
