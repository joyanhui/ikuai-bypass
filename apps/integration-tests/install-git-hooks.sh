#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
hooks_dir="$repo_root/.git/hooks"
mkdir -p "$hooks_dir"

install -m 755 "$repo_root/.githooks/pre-commit" "$hooks_dir/pre-commit"

printf 'Installed pre-commit hook: %s\n' "$hooks_dir/pre-commit"
printf 'It now runs apps/integration-tests/run-smoke-test.sh before each commit.\n'
