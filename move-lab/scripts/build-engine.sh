#!/usr/bin/env bash
# Compiles the battle engine to WebAssembly and copies it into public/engine/.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v emcc >/dev/null 2>&1; then
  env_script="${EMSDK_DIR:-$here/.emsdk}/emsdk_env.sh"
  if [ ! -f "$env_script" ]; then
    echo "Emscripten is missing. Run: npm run setup:emsdk" >&2
    exit 1
  fi
  # shellcheck disable=SC1090
  EMSDK_QUIET=1 source "$env_script"
fi

# Run from the crate folder so its rust-toolchain.toml (which adds the wasm target) applies.
(cd "$here/engine-wasm" && cargo build --release --target wasm32-unknown-emscripten)

built="$here/engine-wasm/target/wasm32-unknown-emscripten/release"
out="$here/public/engine"
mkdir -p "$out"
cp "$built/wbe.js" "$out/wbe.js"
cp "$built/wbe.wasm" "$out/wbe.wasm"

echo "Engine built: $(du -h "$out/wbe.wasm" | cut -f1) wasm, $(du -h "$out/wbe.js" | cut -f1) glue"
