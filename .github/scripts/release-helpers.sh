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

normalize_version_from_tag() {
  local raw="${1:-}"
  if [[ -z "${raw}" ]]; then
    printf '%s' ""
    return
  fi

  if [[ "${raw}" =~ ^ikuai-bypass-v(.+)$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
    return
  fi

  # Why/为什么: 兼容早期 tag 形如 `vX.Y.Z`。
  # English: Keep compatibility with legacy tags like `vX.Y.Z`.
  if [[ "${raw}" =~ ^v(.+)$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
    return
  fi

  printf '%s' "${raw}"
}

is_stable_version() {
  local version="${1:-}"
  [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

is_stable_release_tag() {
  local tag="${1:-}"
  if is_prerelease_tag "${tag}"; then
    return 1
  fi
  local version
  version="$(normalize_version_from_tag "${tag}")"
  is_stable_version "${version}"
}
