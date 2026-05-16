#!/usr/bin/env bash
set -e

# find grammar.js queries/highlights.scm | entr ./build_extension.sh

# --- Configuration ---
# Path to local Zed extension repository
ZED_EXT_DIR="/home/luckybelcik/Documents/vscode/zed/zed-gilt"

echo "==== Building Tree-sitter WASM ===="
tree-sitter build --wasm

echo "==== Syncing files to Zed Extension ===="
cp tree-sitter-gilt.wasm "$ZED_EXT_DIR/grammars/gilt.wasm"

mkdir -p "$ZED_EXT_DIR/languages/gilt"
cp queries/highlights.scm "$ZED_EXT_DIR/languages/gilt/highlights.scm"

echo "Done!"
