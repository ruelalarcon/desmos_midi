#!/bin/bash
if [ ! -f "bin/desmos_midi" ]; then
    echo "Binary not found. Please run './build.sh' first."
    exit 1
fi

./bin/desmos_midi "$@"