#!/bin/bash

# Installation script for Hangar CLI

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}Hangar Installation Script${NC}"
echo "=========================="
echo ""

# Determine installation directory
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_PATH="$SCRIPT_DIR/src-tauri/target/release/hangar"

# Building the project
echo -e "${BLUE}Building the project...${NC}"
cd "$SCRIPT_DIR/src-tauri"
cargo build --release
cd "$SCRIPT_DIR"

if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}❌ Build failed or binary not found at: $BINARY_PATH${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build successful: $BINARY_PATH${NC}"
echo ""

# Check if we need sudo
if [ -w "$INSTALL_DIR" ]; then
    SUDO=""
else
    SUDO="sudo"
    echo -e "${YELLOW}⚠️  Installation requires sudo privileges${NC}"
fi

# Install
echo -e "${BLUE}Installing to: $INSTALL_DIR/hangar${NC}"
$SUDO cp "$BINARY_PATH" "$INSTALL_DIR/hangar"
$SUDO chmod +x "$INSTALL_DIR/hangar"

echo ""
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "Hangar is now installed at: $INSTALL_DIR/hangar"
echo ""
echo "Verify installation:"
echo "  hangar --help"
echo ""
echo "Quick start:"
echo "  hangar serve --daemon --port 8080"
echo "  ./hangar-server.sh status"
echo ""
