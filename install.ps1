$ErrorActionPreference = "Stop"

$BIN_NAME = "ninja-linter.exe"
$REPO = "IgnacioToledoDev/ninja-linter"
$ARCH = "x86_64-pc-windows-msvc"

$URL = "https://github.com/$REPO/releases/latest/download/ninja-linter-$ARCH.zip"
$INSTALL_DIR = "$env:LOCALAPPDATA\ninja-linter"

Write-Host "📦 Installing ninja-linter..."

New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null

$ZIP = "$env:TEMP\ninja-linter.zip"
Invoke-WebRequest $URL -OutFile $ZIP

Expand-Archive $ZIP -DestinationPath $INSTALL_DIR -Force

$BIN_PATH = Join-Path $INSTALL_DIR $BIN_NAME

$PATH_USER = [Environment]::GetEnvironmentVariable("Path", "User")
if ($PATH_USER -notlike "*$INSTALL_DIR*") {
  [Environment]::SetEnvironmentVariable(
    "Path",
    "$PATH_USER;$INSTALL_DIR",
    "User"
  )
}

Write-Host "✅ Installed ninja-linter"
Write-Host "➡️ Restart terminal and run: ninja-linter"
