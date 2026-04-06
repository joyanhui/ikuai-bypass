#!/usr/bin/env bash
set -euo pipefail

workspace_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
artifact_root="${IKB_TEST_ARTIFACT_ROOT:-$workspace_root/apps/integration-tests/.artifacts}"
fixture_bin="${IKB_WEBUI_FIXTURE_BIN:-$workspace_root/target/debug/ikb-webui-fixture}"
frontend_dir="$workspace_root/frontends/app"
launcher_pid=""
env_file=""
json_file=""
launcher_log=""

proxy_unset_if_available() {
  # 对齐用户本机的 proxyUnSet 语义：Bun 前先清空代理环境变量。
  # Mirror the local proxyUnSet alias semantics before invoking Bun.
  unset http_proxy https_proxy all_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY no_proxy NO_PROXY || true
}

cleanup() {
  if [[ -n "$launcher_pid" ]]; then
    kill "$launcher_pid" 2>/dev/null || true
    wait "$launcher_pid" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

mkdir -p "$artifact_root"
env_file="$(mktemp "$artifact_root/webui-browser-env.XXXXXX")"
json_file="$(mktemp "$artifact_root/webui-browser-manifest.XXXXXX.json")"
launcher_log="$(mktemp "$artifact_root/webui-browser-launcher.XXXXXX.log")"

if [[ "${IKB_WEBUI_BROWSER_SKIP_SETUP:-0}" != "1" ]]; then
  printf '==> Building CLI and WebUI fixture binaries\n'
  cargo build --locked -p ikb-cli --bin ikb-cli
  cargo build --locked -p ikb-integration-tests --bin ikb-webui-fixture
fi

if [[ ! -x "$fixture_bin" ]]; then
  printf 'fixture binary not found: %s\n' "$fixture_bin" >&2
  exit 1
fi

if [[ ! -d "$frontend_dir/node_modules/@playwright/test" ]]; then
  printf '==> Installing frontend dependencies\n'
  (
    cd "$frontend_dir"
    proxy_unset_if_available
    bun install --frozen-lockfile
  )
fi

if [[ "${IKB_WEBUI_BROWSER_SKIP_FRONTEND_BUILD:-0}" != "1" ]]; then
  printf '==> Building frontend dist\n'
  (
    cd "$frontend_dir"
    proxy_unset_if_available
    bun run build
  )
fi

if [[ -z "${IKB_PLAYWRIGHT_CHROME_PATH:-}" ]]; then
  IKB_PLAYWRIGHT_CHROME_PATH="$(
    command -v google-chrome \
      || command -v google-chrome-stable \
      || command -v chromium \
      || command -v chromium-browser \
      || true
  )"
  export IKB_PLAYWRIGHT_CHROME_PATH
fi

if [[ -z "${IKB_PLAYWRIGHT_CHROME_PATH:-}" ]]; then
  printf 'no chromium/chrome executable found; set IKB_PLAYWRIGHT_CHROME_PATH\n' >&2
  exit 1
fi

printf '==> Starting simulator-backed WebUI fixture\n'
"$fixture_bin" \
  --env-file "$env_file" \
  --json-file "$json_file" \
  >>"$launcher_log" 2>&1 &
launcher_pid="$!"

for _ in $(seq 1 300); do
  if [[ -s "$env_file" ]]; then
    break
  fi
  if ! kill -0 "$launcher_pid" 2>/dev/null; then
    printf 'webui fixture exited early, log: %s\n' "$launcher_log" >&2
    cat "$launcher_log" >&2 || true
    exit 1
  fi
  sleep 0.1
done

if [[ ! -s "$env_file" ]]; then
  printf 'timed out waiting for webui fixture env file: %s\n' "$env_file" >&2
  printf 'launcher log: %s\n' "$launcher_log" >&2
  exit 1
fi

# 用 source 导入启动器输出的环境变量，供 Playwright 直接复用。
# Source launcher-produced env vars so Playwright can reuse them directly.
set -a
. "$env_file"
set +a

printf '==> Running browser smoke against %s\n' "$IKB_WEBUI_BASE_URL"
(
  cd "$frontend_dir"
  proxy_unset_if_available
  bun run test:e2e -- tests/e2e/webui.smoke.spec.ts "$@"
)
