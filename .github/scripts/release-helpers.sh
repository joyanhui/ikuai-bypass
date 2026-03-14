#!/usr/bin/env bash

# Why/为什么: 将 pre-release 判断逻辑集中在一个函数里，供 workflow 各步骤复用。
# English: Centralize prerelease detection logic for reuse across workflow steps.

is_prerelease_tag() {
  local tag="${1:-}"
  local lc
  lc="$(printf '%s' "${tag}" | tr '[:upper:]' '[:lower:]')"

  # Why/为什么: 这些关键字任意出现即视为 pre-release。
  # English: Treat any tag containing these keywords as prerelease.
  if printf '%s' "${lc}" | grep -Eq '(manuall|manual|test|rc|demo|beta|alpha|pre|preview|dev|nightly)'; then
    return 0
  fi

  return 1
}
