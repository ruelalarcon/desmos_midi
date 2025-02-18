#!/bin/bash
echo "Building in release mode..."
cargo build --release

mkdir -p bin

echo "Copying binary to bin directory..."
cp "target/release/desmos_music" "bin/desmos_music"

echo "Build complete! You can now use './run.sh' to run the program."