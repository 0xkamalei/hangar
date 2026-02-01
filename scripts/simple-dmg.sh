#!/bin/bash

# 简化版 DMG 打包脚本

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  简化 DMG 打包工具  ${NC}"
echo -e "${BLUE}================================${NC}\n"

# 配置
APP_NAME="hangar"
VERSION="0.1.0"
APP_PATH="src-tauri/target/release/bundle/macos/${APP_NAME}.app"
DMG_DIR="src-tauri/target/release/bundle/dmg"
DMG_NAME="${APP_NAME}_${VERSION}_custom_$(uname -m).dmg"
FINAL_DMG="${DMG_DIR}/${DMG_NAME}"

# 检查应用
if [ ! -d "$APP_PATH" ]; then
    echo -e "${RED}错误: 找不到应用包${NC}"
    echo -e "${YELLOW}请先运行: npm run tauri build${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} 找到应用: $APP_PATH"

# 创建临时目录
TMP_DIR=$(mktemp -d)
echo -e "${BLUE}→${NC} 临时目录: $TMP_DIR"

# 复制应用
cp -R "$APP_PATH" "$TMP_DIR/"
echo -e "${GREEN}✓${NC} 应用已复制"

# 创建 Applications 链接
ln -s /Applications "$TMP_DIR/Applications"
echo -e "${GREEN}✓${NC} Applications 链接已创建"

# 创建 README
cat > "$TMP_DIR/安装说明.txt" << EOF
代理订阅管理器 v${VERSION}

=== 安装步骤 ===
1. 将 ${APP_NAME}.app 拖到 Applications 文件夹
2. 打开应用

=== 使用方法 ===
1. 点击"+ 添加订阅"按钮添加您的代理订阅
2. 填写订阅名称和链接
3. 点击"启动服务器"按钮
4. 复制订阅链接: http://127.0.0.1:8080/config
5. 在 Clash Verge 中添加此订阅链接

=== 功能特性 ===
- 多订阅源聚合
- 智能地区分组
- 服务专用组（ChatGPT、Gemini、Netflix 等）
- 启动/停止服务器
- 友好的管理界面

License: MIT
EOF

echo -e "${GREEN}✓${NC} 说明文件已创建"

# 创建 DMG
mkdir -p "$DMG_DIR"
rm -f "$FINAL_DMG"

echo -e "${BLUE}→${NC} 创建 DMG..."

hdiutil create \
    -volname "代理订阅管理器" \
    -srcfolder "$TMP_DIR" \
    -ov \
    -format UDZO \
    -imagekey zlib-level=9 \
    "$FINAL_DMG"

echo -e "${GREEN}✓${NC} DMG 创建完成"

# 清理
rm -rf "$TMP_DIR"
echo -e "${GREEN}✓${NC} 临时文件已清理"

# 结果
echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}  打包完成！${NC}"
echo -e "${GREEN}================================${NC}\n"
echo -e "文件位置: ${BLUE}$FINAL_DMG${NC}"
echo -e "文件大小: ${BLUE}$(du -h "$FINAL_DMG" | cut -f1)${NC}"
echo -e "\n测试命令: ${YELLOW}open \"$FINAL_DMG\"${NC}\n"
