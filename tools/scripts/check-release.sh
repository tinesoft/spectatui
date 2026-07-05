#!/usr/bin/env bash
# Run a spectatui release from CI: version bump, changelog, git commit/tag/push,
# and the GitHub Release — then report to GitHub Actions what happened.
#
# nx release handles version + changelog + git (commit/tag/push) + GitHub
# release. Publishing to crates.io is left to the dedicated `publish` job, so
# `--skip-publish` is always passed. The first ever release has no prior tag to
# diff against, so it needs `--first-release`.
#
# Emits these step outputs (to $GITHUB_OUTPUT) for downstream jobs:
#   released  true|false — whether a new version/tag was created
#   tag       the new tag (e.g. v0.2.0), only set when released=true
#
# Requires: git, pnpm; GITHUB_TOKEN and GITHUB_OUTPUT in the environment.

set -euo pipefail

# Only consider release tags (nx releaseTag pattern is v{version}) — a stray
# non-release tag must not derail first-release detection or the tag output.
describe_release_tag() {
    git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null || true
}

prev_tag="$(describe_release_tag)"

if [ -z "$prev_tag" ]; then
    echo "No existing release tag — running first release."
    pnpm nx release --first-release --skip-publish
else
    echo "Previous release: $prev_tag"
    pnpm nx release --skip-publish
fi

new_tag="$(describe_release_tag)"

if [ -n "$new_tag" ] && [ "$new_tag" != "$prev_tag" ]; then
    echo "Released new version: $new_tag"
    {
        echo "released=true"
        echo "tag=$new_tag"
    } >> "$GITHUB_OUTPUT"
else
    echo "No version bump from these commits — skipping build and publish."
    echo "released=false" >> "$GITHUB_OUTPUT"
fi
