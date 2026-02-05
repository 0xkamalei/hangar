#!/bin/bash

# Hangar Linux Build Script
# This script uses 'cross' (https://github.com/cross-rs/cross) to cross-compile 
# the Hangar CLI for Linux targets, specifically optimized for OpenWrt.

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default target for OpenWrt aarch64
TARGET="${1:-aarch64-unknown-linux-musl}"
OUTPUT_DIR="${2:-dist/linux-${TARGET%%-*}}"

case "$TARGET" in
    aarch64*) ARCH="arm64" ;;
    x86_64*)  ARCH="x86_64" ;;
    *)        ARCH="unknown" ;;
esac

OUTPUT_DIR="dist/linux-$ARCH"

echo -e "${BLUE}Hangar Linux Packaging Tool${NC}"
echo "==========================="
echo ""

cd src-tauri

# Try cargo-zigbuild first if available as it doesn't need Docker
if command -v cargo-zigbuild &> /dev/null && command -v zig &> /dev/null; then
    echo -e "${BLUE}Using cargo-zigbuild for faster compilation...${NC}"
    cargo zigbuild --release --target "$TARGET" --bin hangar --no-default-features
else
    # Check for cross-compilation tool
    if ! command -v cross &> /dev/null; then
        echo -e "${YELLOW}⚠️  'cross' command not found.${NC}"
        echo -e "Installing cross-rs (requires Docker)..."
        cargo install cross --git https://github.com/cross-rs/cross
    fi
    echo -e "${BLUE}Using cross (Docker) for compilation...${NC}"
    cross build --release --target "$TARGET" --bin hangar --no-default-features
fi

cd ..

# Prepare output directory
mkdir -p "$OUTPUT_DIR"

# Copy binary
BINARY_PATH="src-tauri/target/$TARGET/release/hangar"
if [ -f "$BINARY_PATH" ]; then
    cp "$BINARY_PATH" "$OUTPUT_DIR/hangar"
    echo -e ""
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo -e "${GREEN}Binary location: $OUTPUT_DIR/hangar${NC}"
    echo ""
    
    # Create a simple install script for the target device
    cat > "$OUTPUT_DIR/install.sh" << 'EOF'
#!/bin/sh
# Remote installation script for Hangar on OpenWrt/Linux
set -e

INSTALL_PATH="/usr/bin/hangar"

echo "Installing Hangar to $INSTALL_PATH..."
cp ./hangar /usr/bin/hangar
chmod +x /usr/bin/hangar

echo "Hangar installed successfully!"
echo "Try running: hangar --help"
EOF
    chmod +x "$OUTPUT_DIR/install.sh"
    
    # Create a tarball
    TAR_NAME="hangar-linux-arm64.tar.gz"
    tar -czf "$TAR_NAME" -C "dist" "linux-arm64"
    echo -e "${GREEN}✅ Packaged to: $TAR_NAME${NC}"
    echo ""
    echo -e "To install on your OpenWrt device:"
    echo -e "1. scp $TAR_NAME root@your-router-ip:/tmp/"
    echo -e "2. ssh root@your-router-ip 'cd /tmp && tar -xzf $TAR_NAME && cd linux-arm64 && ./install.sh'"

    # Deployment Option
    echo ""
    echo -en "${YELLOW}Do you want to deploy to 192.168.6.1? (y/n): ${NC}"
    # In non-interactive mode, we can just check if we should proceed
    if [[ "$DEPLOY" == "true" ]]; then
        echo "Proceeding with deployment..."
        if command -v sshpass &> /dev/null; then
            echo -e "${BLUE}Deploying to 192.168.6.1:/usr/sbin...${NC}"
            sshpass -p "core" scp -O -o StrictHostKeyChecking=no "$OUTPUT_DIR/hangar" root@192.168.6.1:/usr/sbin/
            sshpass -p "core" ssh -o StrictHostKeyChecking=no root@192.168.6.1 "chmod +x /usr/sbin/hangar"
            echo -e "${GREEN}✅ Deployment successful!${NC}"
        else
            echo -e "${RED}❌ sshpass not found. Skipping deployment.${NC}"
        fi
    fi
else
    echo -e "${RED}❌ Build failed: Binary not found at $BINARY_PATH${NC}"
    exit 1
fi
