#!/usr/bin/env bash

# Why: keep artifact names stable across jobs and platforms / 为什么：统一不同平台产物命名，避免 workflow 到处散落字符串

ikb_normalize_arch() {
  local arch_raw="${1:-}"

  case "${arch_raw}" in
    x86_64)
      printf '%s\n' "x86_64"
      ;;
    i686)
      printf '%s\n' "x86_32"
      ;;
    armv5te)
      printf '%s\n' "arm5"
      ;;
    arm)
      printf '%s\n' "arm6"
      ;;
    armv7)
      printf '%s\n' "arm7"
      ;;
    aarch64|arm64)
      printf '%s\n' "aarch64"
      ;;
    powerpc64le)
      printf '%s\n' "ppc64le"
      ;;
    riscv64gc)
      printf '%s\n' "riscv64gc"
      ;;
    mipsel)
      printf '%s\n' "mipsle"
      ;;
    mips)
      printf '%s\n' "mips"
      ;;
    mips64)
      printf '%s\n' "mips64"
      ;;
    mips64el)
      printf '%s\n' "mips64le"
      ;;
    mipsisa32r6)
      printf '%s\n' "mips"
      ;;
    mipsisa64r6)
      printf '%s\n' "mips64"
      ;;
    mipsisa64r6el)
      printf '%s\n' "mips64le"
      ;;
    *)
      printf '%s\n' "${arch_raw}"
      ;;
  esac
}

ikb_release_arch() {
  local target="${1:-}"
  local arch

  arch="$(ikb_normalize_arch "${target%%-*}")"
  printf '%s\n' "${arch}"
}

ikb_release_os() {
  local target="${1:-}"

  case "${target}" in
    *-windows-*)
      printf '%s\n' "windows"
      ;;
    *-apple-darwin)
      printf '%s\n' "macos"
      ;;
    *-linux-android)
      printf '%s\n' "android"
      ;;
    *-apple-ios)
      printf '%s\n' "ios"
      ;;
    *-freebsd)
      printf '%s\n' "freebsd"
      ;;
    *-linux-*)
      printf '%s\n' "linux"
      ;;
    *)
      printf '%s\n' "unknown"
      ;;
  esac
}

ikb_release_suffix() {
  local target="${1:-}"
  printf '%s-%s\n' "$(ikb_release_os "${target}")" "$(ikb_release_arch "${target}")"
}

ikb_cli_zip_name() {
  local target="${1:-}"
  printf '%s\n' "ikuai-bypass-cli-$(ikb_release_suffix "${target}").zip"
}

ikb_gui_zip_name() {
  local target="${1:-}"
  printf '%s\n' "ikuai-bypass-gui-$(ikb_release_suffix "${target}").zip"
}

ikb_cli_native_name() {
  local target="${1:-}"
  local ext="${2:-}"
  printf '%s\n' "ikuai-bypass-cli-$(ikb_release_suffix "${target}")${ext}"
}

ikb_gui_native_name() {
  local target="${1:-}"
  local ext="${2:-}"
  printf '%s\n' "ikuai-bypass-gui-$(ikb_release_suffix "${target}")${ext}"
}
