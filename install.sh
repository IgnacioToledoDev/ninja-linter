#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="ninja-linter"
REPO="IgnacioToledoDev/ninja-linter"
INSTALL_DIR="/usr/local/bin"

echo "📦 Installing $BIN_NAME..."

# --- Detect OS ---
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS="unknown-linux-gnu" ;;
  *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"
FILENAME="${BIN_NAME}-${TARGET}.zip"

URL="https://github.com/${REPO}/releases/latest/download/${FILENAME}"

echo "⬇️ Downloading $URL"

curl -fsSL "$URL" -o /tmp/$BIN_NAME

chmod +x /tmp/$BIN_NAME
sudo mv /tmp/$BIN_NAME $INSTALL_DIR/$BIN_NAME

echo "✅ Installed $BIN_NAME"
echo "➡️ Run: $BIN_NAME"
