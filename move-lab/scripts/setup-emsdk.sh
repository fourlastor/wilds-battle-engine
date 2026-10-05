#!/usr/bin/env bash
# Downloads the Emscripten SDK into move-lab/.emsdk (git-ignored).
# The version comes from move-lab/emsdk-version so local builds and CI match.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="$(tr -d '[:space:]' < "$here/emsdk-version")"
dir="${EMSDK_DIR:-$here/.emsdk}"

if [ ! -d "$dir/.git" ]; then
  git clone --depth 1 https://github.com/emscripten-core/emsdk.git "$dir"
else
  git -C "$dir" pull --ff-only --depth 1 || true
fi

"$dir/emsdk" install "$version"
"$dir/emsdk" activate "$version"

echo
echo "Emscripten $version is ready in $dir"
echo "scripts/build-engine.sh picks it up automatically."
