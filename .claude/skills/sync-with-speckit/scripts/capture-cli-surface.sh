#!/usr/bin/env bash
# Prints the full `specify` CLI --help command tree to stdout: the top-level
# help plus every subcommand's help, recursing into any subcommand that
# itself lists further subcommands.
#
# Why this is a script and not inline instructions: the subcommand tree is
# discovered live from each --help block's "Commands" box, not from a
# hardcoded list. Spec-Kit adds top-level commands over time (e.g. `bundle`
# didn't exist in earlier releases); a hardcoded list drifts stale the moment
# that happens, silently under-reporting the real CLI surface.
#
# Only ever invokes `--help` — never a mutating `specify` subcommand.
#
# Usage: capture-cli-surface.sh <spec-kit-git-ref>   e.g. v0.13.0
set -euo pipefail

REF="${1:?Usage: $0 <spec-kit-git-ref, e.g. v0.13.0>}"
SRC="git+https://github.com/github/spec-kit.git@${REF}"
MAX_DEPTH=6

run_help() {
  uvx --from "$SRC" specify "$@" --help 2>/dev/null
}

# Extracts subcommand names from a --help block's "Commands" box.
# A real "name  description" row has exactly one space after the box border
# before the name starts; a wrapped continuation line of a multi-line
# description is padded with many spaces to align under the description
# column (no name in that column). Distinguishing these matters: without it,
# words from a wrapped description (e.g. "...preview upgrades..." wrapping
# under `self`) get misread as command names that don't exist.
extract_subcommands() {
  awk '
    /^╭─ Commands/ { in_commands=1; next }
    /^╰/ { in_commands=0 }
    in_commands && /^│/ {
      line = $0
      sub(/^│/, "", line)
      if (line ~ /^ [^ ]/) {
        sub(/^ /, "", line)
        n = split(line, parts, " ")
        if (n > 0) print parts[1]
      }
    }
  '
}

visit() {
  local depth="$1"; shift
  local path=("$@")
  local label="specify${path[*]:+ ${path[*]}}"

  if (( depth > MAX_DEPTH )); then
    echo "=== $label --help === (SKIPPED: exceeded max recursion depth $MAX_DEPTH — capture manually if this level matters)"
    echo
    return
  fi

  echo "=== $label --help ==="
  local out
  if ! out="$(run_help "${path[@]}")"; then
    echo "(FAILED: '$label --help' exited non-zero — could not capture this branch, continuing)"
    echo
    return
  fi
  echo "$out"
  echo

  local sub
  while IFS= read -r sub; do
    [[ -z "$sub" ]] && continue
    visit "$((depth + 1))" "${path[@]}" "$sub"
  done <<< "$(echo "$out" | extract_subcommands)"
}

visit 0
