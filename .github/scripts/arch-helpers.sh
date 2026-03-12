#!/usr/bin/env bash

# Why: keep artifact names stable across jobs and platforms / 为什么：统一不同平台产物命名，避免 workflow 到处散落字符串

ikb_normalize_arch() {
  local arch_raw="${1:-}"

  case "${arch_raw}" in
    x86_64)
      printf '%s\n' "x86_64"
      ;;
    i686)
      printf '%s\n' "x86"
      ;;
    aarch64|arm64)
      printf '%s\n' "aarch64"
      ;;
    *)
      printf '%s\n' "${arch_raw}"
      ;;
  esac
}

ikb_detect_os() {
  local target="${1:-}"

  if [[ "${target}" == *windows* ]]; then
    printf '%s\n' "windows"
  elif [[ "${target}" == *apple-darwin* ]]; then
    printf '%s\n' "macos"
  elif [[ "${target}" == *freebsd* ]]; then
    printf '%s\n' "freebsd"
  elif [[ "${target}" == *android* ]]; then
    printf '%s\n' "android"
  elif [[ "${target}" == *linux* ]]; then
    printf '%s\n' "linux"
  else
    printf '%s\n' "unknown"
  fi
}

ikb_detect_libc() {
  local target="${1:-}"

  if [[ "${target}" == *musl* ]]; then
    printf '%s\n' "musl"
  elif [[ "${target}" == *gnu* ]]; then
    printf '%s\n' "gnu"
  elif [[ "${target}" == *msvc* ]]; then
    printf '%s\n' "msvc"
  elif [[ "${target}" == *freebsd* ]]; then
    printf '%s\n' "freebsd"
  elif [[ "${target}" == *darwin* ]]; then
    printf '%s\n' "darwin"
  else
    printf '%s\n' "native"
  fi
}

ikb_cli_base() {
  local target="${1:-}"
  local arch

  arch="$(ikb_normalize_arch "${target%%-*}")"
  printf '%s\n' "ikuai-bypass-cli-${arch}-$(ikb_detect_os "${target}")-$(ikb_detect_libc "${target}")"
}

ikb_gui_base() {
  local target="${1:-}"
  local arch

  arch="$(ikb_normalize_arch "${target%%-*}")"
  printf '%s\n' "ikuai-bypass-gui-${arch}-$(ikb_detect_os "${target}")-$(ikb_detect_libc "${target}")"
}
