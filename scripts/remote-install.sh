#!/bin/sh

set -e

REPO_URL="https://github.com/decisivestrike/chameleon.git"

command -v git >/dev/null 2>&1 || error "git is required but not installed."
command -v cargo >/dev/null 2>&1 || error "cargo is required but not installed."

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

echo "Downloading to temp directory: $TEMP_DIR"
git clone --depth 1 "$REPO_URL" "$TEMP_DIR/chameleon" || error "Failed to clone repository."

bash "$TEMP_DIR/chameleon/scripts/install.sh"