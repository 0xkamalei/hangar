#!/bin/bash

# 快速构建脚本 - 一键构建并打包 DMG

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  快速构建和打包工具  ${NC}"
echo -e "${BLUE}================================${NC}\n"

# 1. TypeScript 类型检查
echo -e "${YELLOW}[1/5]${NC} TypeScript 类型检查..."
bun run tsc --noEmit
echo -e "${GREEN}✓${NC} TypeScript 类型检查通过\n"

# 2. Rust 测试
echo -e "${YELLOW}[2/5]${NC} 运行 Rust 测试..."
cargo test --manifest-path src-tauri/Cargo.toml --quiet
echo -e "${GREEN}✓${NC} Rust 测试通过\n"

# 3. Clippy 检查
echo -e "${YELLOW}[3/5]${NC} Clippy 代码检查..."
cargo clippy --manifest-path src-tauri/Cargo.toml --quiet
echo -e "${GREEN}✓${NC} Clippy 检查通过\n"

# 4. 构建应用
echo -e "${YELLOW}[4/5]${NC} 构建应用..."
bun run tauri build -- --verbose | tail -20
echo -e "${GREEN}✓${NC} 应用构建完成\n"

# 5. 创建 DMG
echo -e "${YELLOW}[5/5]${NC} 创建 DMG 安装包...\n"
./scripts/build-dmg.sh

echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}  构建流程全部完成！${NC}"
echo -e "${GREEN}================================${NC}\n"
