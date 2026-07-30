#!/usr/bin/env bash
# Fails if a crate's Cargo.toml version lags behind the highest Phase in its
# TODO.md whose checkboxes are all checked off.
#
# Each crate's TODO.md marks phases like `## Phase N — Name `[X.Y.Z]``. When
# every `- [ ]` under a phase becomes `- [x]`, that phase's target version is
# considered released and Cargo.toml must be bumped to match (with a
# corresponding CHANGELOG.md entry) before the phase is merged.
set -euo pipefail

cd "$(dirname "$0")/.."

fail=0

for todo in crates/*/TODO.md; do
  crate_dir="$(dirname "$todo")"
  cargo_toml="$crate_dir/Cargo.toml"
  [ -f "$cargo_toml" ] || continue

  cargo_version="$(grep -m1 '^version = "' "$cargo_toml" | sed -E 's/version = "([^"]+)"/\1/')"
  [ -n "$cargo_version" ] || continue

  mapfile -t headers < <(grep -n '^## Phase [0-9]\+ .*`\[[0-9]\+\.[0-9]\+\.[0-9]\+\]`' "$todo")
  total_lines=$(wc -l < "$todo")

  highest_complete=""
  for i in "${!headers[@]}"; do
    entry="${headers[$i]}"
    line_no="${entry%%:*}"
    target_version="$(echo "$entry" | grep -oE '\[[0-9]+\.[0-9]+\.[0-9]+\]' | tr -d '[]')"
    if [ $((i + 1)) -lt "${#headers[@]}" ]; then
      next_header_line="${headers[$((i + 1))]%%:*}"
      end_line=$((next_header_line - 1))
    else
      end_line="$total_lines"
    fi

    incomplete=0
    if sed -n "${line_no},${end_line}p" "$todo" | grep -q '^[[:space:]]*- \[ \]'; then
      incomplete=1
    fi

    if [ "$incomplete" -eq 0 ]; then
      highest_complete="$target_version"
    fi
  done

  [ -n "$highest_complete" ] || continue

  newest="$(printf '%s\n%s\n' "$cargo_version" "$highest_complete" | sort -V | tail -1)"
  if [ "$newest" != "$cargo_version" ]; then
    echo "version-todo-sync: $todo shows phase [$highest_complete] fully checked off, but $cargo_toml version is $cargo_version" >&2
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  echo "Bump the crate version (and add a matching CHANGELOG.md entry) to close out a completed TODO.md phase before committing." >&2
  exit 1
fi
