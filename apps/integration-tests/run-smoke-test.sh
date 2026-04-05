#!/usr/bin/env bash
set -euo pipefail

default_tests=(
  simulator_api_surface_smoke
  rule_sync_update_in_place_smoke
  safe_before_smoke
  clean_mode_smoke
  clean_all_smoke
  export_stream_domain_smoke
  cli_modes_smoke
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
backend="${IKB_TEST_BACKEND:-auto}"
default_image_path="$workspace_root/.github/ikuai.qcow2"
default_image_archive="$workspace_root/.github/ikuai.qcow2.7z"

mkdir -p "$artifact_root"

if [ -z "${IKB_TEST_IKUAI_IMAGE:-}" ] && [ "$backend" != "simulator" ] && [ "$backend" != "sim" ]; then
  if command -v qemu-system-x86_64 >/dev/null 2>&1; then
    if [ ! -f "$default_image_path" ] && [ -f "$default_image_archive" ] && command -v 7z >/dev/null 2>&1; then
      printf '==> Extracting default iKuai image from %s\n' "$default_image_archive"
      7z x "$default_image_archive" "-o$workspace_root/.github" >/dev/null
    fi
    if [ -f "$default_image_path" ]; then
      export IKB_TEST_IKUAI_IMAGE="$default_image_path"
    fi
  fi
fi

printf '==> Building CLI test binary\n'
if [ "${IKB_SMOKE_SKIP_SETUP:-0}" != "1" ]; then
  cargo build --locked -p ikb-cli --bin ikb-cli

  printf '\n==> Running core unit tests\n'
  cargo test --locked -p ikb-core
else
  printf '==> Skipping setup (IKB_SMOKE_SKIP_SETUP=1)\n'
fi

for test_name in "${test_names[@]}"; do
  printf '\n==> Running %s\n' "$test_name"
  IKB_TEST_DEV_SCRIPT="$dev_script_path" \
    IKB_TEST_ARTIFACT_ROOT="$artifact_root" \
    RUST_TEST_THREADS=1 \
    cargo test --locked -p ikb-integration-tests --test "$test_name" -- --nocapture "${cargo_test_args[@]}"
done
