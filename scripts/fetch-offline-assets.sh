#!/usr/bin/env bash
# Fetch the assets embedded into the "offline" build of the assistant:
#   - Node.js 22 LTS (matching the target OS/arch)
#   - openlearn-next npm tarball
# Results are written into src-tauri/resources/ with fixed names so the
# Rust `offline` feature can include_bytes! them.
#
# Usage: ./scripts/fetch-offline-assets.sh <os> <arch>
#   os   : linux | macos | windows
#   arch : x86_64 | aarch64
set -euo pipefail

OS="${1:-linux}"
ARCH="${2:-x86_64}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RES="$ROOT/src-tauri/resources"
mkdir -p "$RES"

NODE_VER="v22.14.0"

node_asset() {
  case "$OS-$ARCH" in
    linux-x86_64)   echo "node-${NODE_VER}-linux-x64.tar.gz";;
    linux-aarch64)  echo "node-${NODE_VER}-linux-arm64.tar.gz";;
    macos-x86_64)   echo "node-${NODE_VER}-darwin-x64.tar.gz";;
    macos-aarch64)  echo "node-${NODE_VER}-darwin-arm64.tar.gz";;
    windows-x86_64) echo "node-${NODE_VER}-win-x64.zip";;
    *) echo "unsupported target: $OS-$ARCH" >&2; exit 1;;
  esac
}

ASSET="$(node_asset)"
IS_ZIP=0
[[ "$ASSET" == *.zip ]] && IS_ZIP=1

echo ">> Downloading Node $NODE_VER ($ASSET)"
curl -fSL "https://nodejs.org/dist/${NODE_VER}/${ASSET}" -o "$RES/node-asset.$([ "$IS_ZIP" = 1 ] && echo zip || echo tar.gz)"

echo ">> Fetching openlearn-next tarball"
TMP="$(mktemp -d)"
( cd "$TMP" && npm pack openlearn-next >/dev/null )
TGZ="$(ls "$TMP"/openlearn-next-*.tgz | head -n1)"
cp "$TGZ" "$RES/openlearn-next.tgz"
rm -rf "$TMP"

echo ">> Assets written to $RES"
ls -lh "$RES"
