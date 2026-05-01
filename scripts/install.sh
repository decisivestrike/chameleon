#!/bin/sh

set -e

REPO_URL="https://github.com/decisivestrike/chameleon.git"
BRANCH="${1:-main}"
INSTALL_DIR="/usr/local/bin"
CHAMELEON_ROOT="$HOME/.local/chameleon"

mkdir -p "$CHAMELEON_ROOT/bin"
cd "$CHAMELEON_ROOT"

git clone --depth 1 --branch "$BRANCH" "$REPO_URL" repo

if ! command -v go >/dev/null 2>&1; then
    echo "Ошибка: Go не установлен. Установите его и повторите попытку."
    exit 1
fi

cd "repo/cli"
go build -o chameleon

echo "Installing to /usr/local/bin"
sudo mv chameleon /usr/local/bin

echo "Done"