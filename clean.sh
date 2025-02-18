#!/bin/bash
echo "Cleaning build artifacts..."

if [ -d "bin" ]; then
    echo "Removing bin directory..."
    rm -rf bin
fi

if [ -d "target" ]; then
    echo "Removing target directory..."
    rm -rf target
fi

echo "Clean complete!"