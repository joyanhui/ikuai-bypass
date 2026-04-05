#!/usr/bin/env bash
set -euo pipefail

default_tests=(
  rule_sync_update_in_place_smoke
  safe_before_smoke
  clean_mode_smoke
)

test_names=()
cargo_test_args=()
parse_test_names=true

for arg in "$@"; do
  if [ "$arg" = "--" ]; then
    parse_test_names=false
    continue
  fi

  if [ "$parse_test_names" = "true" ]; then
    test_names+=("$arg")
  else
    cargo_test_args+=("$arg")
  fi
done

if [ ${#test_names[@]} -eq 0 ]; then
  test_names=("${default_tests[@]}")
fi

workspace_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
artifact_root="${IKB_TEST_ARTIFACT_ROOT:-$workspace_root/apps/integration-tests/.artifacts}"
dev_script_path="${IKB_TEST_DEV_SCRIPT:-$workspace_root/script/dev.sh}"

mkdir -p "$artifact_root"

printf '==> Building CLI test binary\n'
cargo build --locked -p ikb-cli --bin ikb-cli

printf '\n==> Running core unit tests\n'
cargo test --locked -p ikb-core

for test_name in "${test_names[@]}"; do
  printf '\n==> Running %s\n' "$test_name"
  IKB_TEST_DEV_SCRIPT="$dev_script_path" \
    IKB_TEST_ARTIFACT_ROOT="$artifact_root" \
    RUST_TEST_THREADS=1 \
    cargo test --locked -p ikb-integration-tests --test "$test_name" -- --nocapture "${cargo_test_args[@]}"
done
