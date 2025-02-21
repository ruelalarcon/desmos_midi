#!/bin/bash
echo "Building in release mode..."
cargo build --release

mkdir -p bin

echo "Copying binary to bin directory..."
cp "target/release/desmos_midi" "bin/desmos_midi"

echo "Build complete! You can now use './run.sh' to run the program."