#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MACOS_ICON="$ROOT_DIR/src/assets/icon_1024.icns"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This packaging script only supports macOS." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but was not found in PATH." >&2
  exit 1
fi

if ! cargo packager --version >/dev/null 2>&1; then
  echo "cargo-packager is not installed. Installing it now..."
  cargo install cargo-packager --locked
fi

cd "$ROOT_DIR"

export CLANG_MODULE_CACHE_PATH="${CLANG_MODULE_CACHE_PATH:-$ROOT_DIR/target/clang-module-cache}"
mkdir -p "$CLANG_MODULE_CACHE_PATH"

if [[ ! -f "$MACOS_ICON" ]]; then
  echo "Missing macOS icon: $MACOS_ICON" >&2
  exit 1
fi

echo "Packaging MDRS as a macOS DMG installer..."
cargo packager --release --formats dmg

echo
echo "Done. Check dist/packager/ for the generated macOS installer."
