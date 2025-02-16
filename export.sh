#!/bin/bash
if [ -z "$1" ]; then
    echo "Usage: ./export.sh <midi_file>"
    echo "Exports a MIDI file to timing.bin and formulas.txt"
    exit 1
fi
cargo run --release -- export "$1"