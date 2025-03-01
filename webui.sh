#!/bin/bash

if [ ! -f "target/release/desmos_midi_web" ]; then
    echo "Binary not found. Please run './build.sh' first."
    exit 1
fi

# Start the server with all arguments passed through
./target/release/desmos_midi_web "$@"