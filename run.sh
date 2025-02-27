#!/bin/bash
if [ ! -f "target/release/desmos_midi" ]; then
    echo "Binary not found. Please run './build.sh' first."
    exit 1
fi

./target/release/desmos_midi "$@"