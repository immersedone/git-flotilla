#!/bin/bash
# ============================================================================
# Build Git Flotilla Windows artifacts from WSL2
# ============================================================================
#
# This script syncs the source code to a Windows-accessible directory and
# builds the Tauri application using Windows-native toolchains via PowerShell.
#
# Prerequisites (installed on the Windows side):
#   - Rust (MSVC toolchain): https://rustup.rs
#   - Node.js 20+: https://nodejs.org
#   - MSVC Build Tools (Visual Studio 2022 Build Tools with "Desktop development with C++")
#   - NSIS (for .exe installer): https://nsis.sourceforge.io
#   - WebView2 (usually pre-installed on Windows 10/11)
#
# Usage:
#   ./scripts/build-windows.sh
#
# Output:
#   - NSIS installer: C:\git-flotilla-build\src-tauri\target\release\bundle\nsis\*.exe
#   - MSI installer:  C:\git-flotilla-build\src-tauri\target\release\bundle\msi\*.msi
#
# ============================================================================

set -e

BUILD_DIR="/mnt/c/git-flotilla-build"
SOURCE_DIR="/var/www/vhosts/git-flotilla"

# ---------------------------------------------------------------------------
# Adjust these paths to match your Windows environment.
# Run `where rustc`, `where node`, `where makensis` in a Windows terminal
# to find the correct paths.
# ---------------------------------------------------------------------------
VCVARS='"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"'
RUST_PATH='C:\Program Files\Rust stable MSVC 1.94\bin'
NODE_PATH='C:\Program Files\nodejs'
NSIS_PATH='C:\Program Files (x86)\NSIS'
PATHS="${RUST_PATH};${NODE_PATH};${NSIS_PATH}"

echo "=== Git Flotilla Windows Build ==="
echo "Source:  ${SOURCE_DIR}"
echo "Build:   ${BUILD_DIR}"
echo ""

# ---------------------------------------------------------------------------
# Step 1: Sync source to Windows-accessible directory
# ---------------------------------------------------------------------------
echo "=== Syncing source to Windows build directory ==="
rsync -av --delete \
  --exclude='node_modules' \
  --exclude='target' \
  --exclude='dist' \
  --exclude='.git' \
  --exclude='.worktrees' \
  --exclude='.flotilla/cache' \
  "$SOURCE_DIR/" "$BUILD_DIR/"

echo ""

# ---------------------------------------------------------------------------
# Step 2: Install pnpm and frontend dependencies
# ---------------------------------------------------------------------------
echo "=== Installing dependencies via PowerShell ==="
powershell.exe -Command "cmd /c '${VCVARS} >nul 2>&1 && set PATH=${PATHS};%PATH% && cd /d C:\git-flotilla-build && npm install -g pnpm && pnpm install'"

echo ""

# ---------------------------------------------------------------------------
# Step 3: Build Tauri application
# ---------------------------------------------------------------------------
echo "=== Building Tauri app via PowerShell ==="
powershell.exe -Command "cmd /c '${VCVARS} >nul 2>&1 && set PATH=${PATHS};%PATH% && cd /d C:\git-flotilla-build && npx tauri build'"

echo ""

# ---------------------------------------------------------------------------
# Step 4: Report results
# ---------------------------------------------------------------------------
echo "=== Build complete ==="
echo ""
echo "NSIS installer:"
ls -lh "$BUILD_DIR/src-tauri/target/release/bundle/nsis/"*.exe 2>/dev/null || echo "  (not found)"
echo ""
echo "MSI installer:"
ls -lh "$BUILD_DIR/src-tauri/target/release/bundle/msi/"*.msi 2>/dev/null || echo "  (not found)"
echo ""
echo "To upload to a GitHub release:"
echo "  gh release upload v0.1.0 \"$BUILD_DIR/src-tauri/target/release/bundle/nsis/Git Flotilla_0.1.0_x64-setup.exe\" --clobber"
echo "  gh release upload v0.1.0 \"$BUILD_DIR/src-tauri/target/release/bundle/msi/Git Flotilla_0.1.0_x64_en-US.msi\" --clobber"
