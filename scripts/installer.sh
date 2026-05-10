#!/bin/sh

set -e

REPO_URL="https://github.com/decisivestrike/chameleon.git"
CRATES=("chameleon-launcher" "chameleon-notifications" "chameleon-panel" "chameleon-widgets")
TARGET_DIR="$HOME/.chameleon/bin"
mkdir -p "$TARGET_DIR"

command -v git >/dev/null 2>&1 || error "git is required but not installed."
command -v cargo >/dev/null 2>&1 || error "cargo is required but not installed."

WORK_DIR=$(mktemp -d)
log "Working directory: $WORK_DIR"

git clone --depth 1 "$REPO_URL" "$WORK_DIR/chameleon" || error "Failed to clone repository."
cd "$WORK_DIR/chameleon"

for crate in "${CRATES[@]}"; do
    echo "Building $crate ..."
    (cd "crates/$crate" && cargo build --release)
    cp "target/release/$crate" "$TARGET_DIR/"
done

echo "Done"