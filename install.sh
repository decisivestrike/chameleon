#!/bin/bash

set -e

BINARY_NAME="chameleon"
TARGET_BIN="target/release/$BINARY_NAME"
INSTALL_PATH="/usr/local/bin/$BINARY_NAME"

echo "Starting the build in release mode..."
cargo build --release

if [ ! -f "$TARGET_BIN" ]; then
    echo "Error: Binary $TARGET_BIN not found after build."
    exit 1
fi

if [ -f "$INSTALL_PATH" ]; then
    echo "File $INSTALL_PATH already exists. Overwrite? (y/N)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    echo "Overwriting $INSTALL_PATH..."
else
    echo "Installing to $INSTALL_PATH..."
fi

sudo cp "$TARGET_BIN" "$INSTALL_PATH"

echo "Done"