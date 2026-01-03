#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="ninja-linter"
REPO="IgnacioToledoDev/ninja-linter"
INSTALL_DIR="/usr/local/bin"

echo "📦 Installing $BIN_NAME..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  Linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
  Darwin) TARGET="${ARCH}-apple-darwin" ;;
  *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

FILENAME="${BIN_NAME}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${FILENAME}"

TMP_DIR="$(mktemp -d)"

echo "⬇️ Downloading $URL"
curl -fsSL "$URL" -o "$TMP_DIR/$FILENAME"

echo "📂 Extracting..."
tar -xzf "$TMP_DIR/$FILENAME" -C "$TMP_DIR"

chmod +x "$TMP_DIR/$BIN_NAME"
sudo install "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

echo "✅ Installed $BIN_NAME"
echo "➡️ Run: $BIN_NAME"
