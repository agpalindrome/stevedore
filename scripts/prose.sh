#!/usr/bin/env bash
#
# The prose gate. Lints this repo's markdown against the house style vendored in
# .vale/styles, which ~/.claude/scripts/sync-vale.sh writes here — vendored
# rather than referenced so CI needs no access to that private repo.
#
# Errors block, warnings do not. vale exits non-zero on errors only, so that
# needs no flag; a gate that fails on a warning backlog is a gate that gets
# switched off.
#
# Both .github/workflows/ci.yml and .githooks/pre-push call this, so the local
# verdict is the CI verdict — the same property the clippy gate has here.
#
# Requires: vale (supplied by the flake dev shell).
set -euo pipefail

command -v vale >/dev/null 2>&1 || {
  echo "error: 'vale' not found on PATH; enter the dev shell" >&2
  exit 1
}

cd "$(git rev-parse --show-toplevel)"

# Every tracked markdown file. All of them are stevedore's own documentation:
# there are no prose fixtures, and nothing here is written in a voice that is
# the author's rather than the project's. Narrow this when one appears.
files=$(git ls-files '*.md')

# Assert the gate has inputs. Given an empty list, `xargs vale` runs vale with
# no paths, so it lints empty stdin, prints "0 errors" and exits 0 — a green
# check over nothing. An empty list here means this glob broke, never that the
# repo has no prose.
count=$(printf '%s\n' "$files" | grep -c . || true)
if [ "$count" -eq 0 ]; then
  echo "error: no markdown matched — the file list is broken, not the prose" >&2
  exit 1
fi

echo "prose: $(vale --version) over $count markdown file(s)"

# --no-global: without it vale merges a machine-global styles directory over the
# vendored rules, which is exactly how a local run comes to disagree with CI.
printf '%s\n' "$files" | xargs vale --no-global --config .vale.ini
