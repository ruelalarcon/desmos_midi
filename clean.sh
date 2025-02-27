#!/bin/bash
echo "Cleaning build artifacts..."

if [ -d "target" ]; then
    echo "Removing target directory..."
    rm -rf target
fi

echo "Clean complete!"