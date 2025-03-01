#!/bin/bash

show_help() {
    echo "Usage: build.sh [OPTIONS]"
    echo
    echo "Options:"
    echo "  --cli-only    Build only the CLI version"
    echo "  --web-only    Build only the Web UI version"
    echo "  -h, --help    Show this help message"
    echo
    echo "If no options are provided, both CLI and Web UI will be built."
    exit 1
}

# Default to building everything
BUILD_CLI=1
BUILD_WEB=1

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --cli-only)
            BUILD_WEB=0
            shift
            ;;
        --web-only)
            BUILD_CLI=0
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            echo "ERROR: Unknown argument: $1"
            show_help
            ;;
    esac
done

echo "Building in release mode..."

# Build based on selected options
if [ $BUILD_CLI -eq 1 ]; then
    if [ $BUILD_WEB -eq 1 ]; then
        echo "Building CLI and Web UI..."
        cargo build --release --features "cli webui" --bins
    else
        echo "Building CLI..."
        cargo build --release --features cli --bin desmos_midi
    fi
else
    if [ $BUILD_WEB -eq 1 ]; then
        echo "Building Web UI..."
        cargo build --release --features webui --bin desmos_midi_web
    fi
fi

if [ $? -ne 0 ]; then
    echo "ERROR: Build failed! Please check the error messages above."
    exit 1
fi

echo "Build completed successfully!"
if [ $BUILD_CLI -eq 1 ]; then
    echo "You can use './cli.sh' to run the CLI."
fi
if [ $BUILD_WEB -eq 1 ]; then
    echo "You can use './webui.sh' to run the Web UI."
fi