#!/bin/bash
echo "Building in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed! Please check the error messages above."
    exit 1
fi

echo "Build complete! You can now use './run.sh' to run the program."