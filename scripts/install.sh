#!/bin/sh

set -e

install() {
    cargo install --path "$1" --root "$CHAMELEON_ROOT" --no-track --force
}

CHAMELEON_ROOT="$HOME/.chameleon"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "$CHAMELEON_ROOT/bin"
echo "Installing modules to $CHAMELEON_ROOT/bin"
cd "$SCRIPT_DIR"
cd ../crates

for module in "chameleon" "chameleon-launcher" "chameleon-notifications" "chameleon-panel"; do
    echo "Installing $module..."
    install "./$module"
done

cp -r ../assets "$CHAMELEON_ROOT"

echo "Done"