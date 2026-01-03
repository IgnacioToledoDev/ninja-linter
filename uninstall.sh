#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="ninja-linter"
INSTALL_DIR="/usr/local/bin"

echo "🗑️ Uninstalling $BIN_NAME..."

if [ -f "$INSTALL_DIR/$BIN_NAME" ]; then
    sudo rm "$INSTALL_DIR/$BIN_NAME"
    echo "✅ $BIN_NAME has been removed from $INSTALL_DIR"
else
    echo "❓ $BIN_NAME is not installed in $INSTALL_DIR"
fi
