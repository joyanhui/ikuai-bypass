#!/usr/bin/env bash
# ── iKuai Bypass Docs — Local Jekyll Development Script ──
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

cd "$SCRIPT_DIR"

if ! command -v ruby >/dev/null 2>&1; then
    echo "Error: Ruby is not installed."
    echo "  Ubuntu/Debian: sudo apt install ruby-full"
    echo "  NixOS: nix shell nixpkgs#ruby nixpkgs#bundler"
    exit 1
fi

if ! command -v bundle >/dev/null 2>&1; then
    echo "Installing Bundler..."
    gem install bundler
fi

bundle install

echo "Starting Jekyll dev server at http://127.0.0.1:4000"
exec bundle exec jekyll serve \
    --baseurl '' \
    --watch \
    --livereload \
    --host 127.0.0.1 \
    --port 4000
