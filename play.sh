#!/bin/bash
if [ -z "$1" ]; then
    echo "Usage: ./play.sh <song_folder>"
    echo "Plays a song from the specified folder"
    exit 1
fi
cargo run --release -- play "$1"