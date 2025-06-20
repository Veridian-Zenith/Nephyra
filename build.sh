#!/bin/sh
# Build Nephyra in release mode and copy the binary to ~/bin (user-local bin dir)
set -e
cargo build --release
BIN_DIR="$HOME/bin"
RELEASE_BIN="$(pwd)/target/release/Nephyra"
mkdir -p "$BIN_DIR"
cp "$RELEASE_BIN" "$BIN_DIR/Nephyra"
echo "Build complete. Binary is at $BIN_DIR/Nephyra"
