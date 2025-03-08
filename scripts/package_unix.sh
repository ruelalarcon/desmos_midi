#!/bin/bash
# Script to package the Desmos MIDI application for Unix/Linux systems
# Creates a zip file with the binary, soundfonts folder, and config.toml

# Exit on first error
set -e

# Check if zip is installed
if ! command -v zip &> /dev/null; then
    echo "Error: 'zip' command is not installed."
    echo "Please install it using your package manager:"
    echo ""
    echo "  Debian/Ubuntu: sudo apt-get install zip"
    echo "  Fedora/RHEL:   sudo dnf install zip"
    echo "  Arch Linux:    sudo pacman -S zip"
    echo "  macOS:         brew install zip"
    echo ""
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(grep -m 1 'version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Create package name with platform and architecture info
PACKAGE_NAME="desmos_midi_${VERSION}_${PLATFORM}_${ARCH}"
OUTPUT_DIR="package"
ZIP_FILE="${PACKAGE_NAME}.zip"

echo "Building release binaries..."
cargo build --release

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"
PACKAGE_DIR="${OUTPUT_DIR}/${PACKAGE_NAME}"
rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

# Copy binary files
echo "Copying binary files..."
cp "target/release/desmos_midi" "$PACKAGE_DIR"
cp "target/release/desmos_midi_web" "$PACKAGE_DIR"

# Copy config and soundfonts
echo "Copying configuration and soundfonts..."
cp "config.toml" "$PACKAGE_DIR"
cp -r "soundfonts" "$PACKAGE_DIR"

# Copy README and LICENSE
cp "README.md" "$PACKAGE_DIR"
cp "LICENSE.txt" "$PACKAGE_DIR"

# Create zip file
echo "Creating zip file..."
ZIP_PATH="${OUTPUT_DIR}/${ZIP_FILE}"
rm -f "$ZIP_PATH"
(cd "$OUTPUT_DIR" && zip -r "$ZIP_FILE" "$PACKAGE_NAME")

echo "Package created at: $ZIP_PATH"

# Make the binaries executable
chmod +x "${PACKAGE_DIR}/desmos_midi"
chmod +x "${PACKAGE_DIR}/desmos_midi_web"