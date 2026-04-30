#!/bin/bash

set -e

BINARY_NAME="chameleon"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $SCRIPT_DIR
echo "Building..."
cd ../cli && go build

if [ -f "$BINARY_NAME" ]; then
    echo "Installing to /usr/local/bin (requires sudo)..."
    sudo mv "$BINARY_NAME" /usr/local/bin/
    echo "Done"
else
    echo "Build failed: binary not found."
    exit 1
fi
