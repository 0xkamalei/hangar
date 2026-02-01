#!/bin/bash

# DMG 打包脚本 - 代理订阅管理器
# 用于创建美观的 macOS DMG 安装镜像

set -e

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  代理订阅管理器 DMG 打包工具  ${NC}"
echo -e "${BLUE}================================${NC}\n"

# 配置变量
APP_NAME="proxy-sub-manager"
APP_DISPLAY_NAME="代理订阅管理器"
VERSION="0.1.0"
BUNDLE_DIR="src-tauri/target/release/bundle"
APP_PATH="${BUNDLE_DIR}/macos/${APP_NAME}.app"
DMG_DIR="${BUNDLE_DIR}/dmg"
DMG_NAME="${APP_NAME}_${VERSION}_$(uname -m).dmg"
DMG_PATH="${DMG_DIR}/${DMG_NAME}"

# 临时目录
TMP_DMG_DIR="/tmp/${APP_NAME}_dmg"
VOLUME_NAME="${APP_DISPLAY_NAME}"

# 检查应用是否存在
if [ ! -d "$APP_PATH" ]; then
    echo -e "${RED}错误: 找不到应用包${NC}"
    echo -e "${YELLOW}请先运行: npm run tauri build${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} 找到应用包: $APP_PATH"

# 清理旧的临时文件
echo -e "${BLUE}→${NC} 清理临时文件..."
rm -rf "$TMP_DMG_DIR"
mkdir -p "$TMP_DMG_DIR"
mkdir -p "$DMG_DIR"

# 复制应用到临时目录
echo -e "${BLUE}→${NC} 复制应用..."
cp -R "$APP_PATH" "$TMP_DMG_DIR/"

# 创建 Applications 文件夹的符号链接
echo -e "${BLUE}→${NC} 创建 Applications 符号链接..."
ln -s /Applications "$TMP_DMG_DIR/Applications"

# 创建自定义图标位置脚本
cat > "$TMP_DMG_DIR/.DS_Store_template" << 'EOF'
# DMG 窗口布局配置
# 这个文件会被下面的 AppleScript 使用
EOF

# 创建 README 文件
cat > "$TMP_DMG_DIR/README.txt" << EOF
${APP_DISPLAY_NAME} v${VERSION}

安装说明：
1. 将 ${APP_NAME}.app 拖到 Applications 文件夹
2. 打开应用
3. 在界面中添加您的代理订阅
4. 点击"启动服务器"

使用说明：
- 添加订阅：点击"+ 添加订阅"按钮
- 编辑订阅：点击订阅卡片上的"编辑"按钮
- 启用/禁用：点击订阅卡片上的"启用/禁用"按钮
- 启动服务器：点击"启动服务器"按钮
- 订阅链接：http://127.0.0.1:8080/config

在 Clash Verge 中添加上述订阅链接即可使用。

项目地址: https://github.com/yourusername/proxy-sub-manager
License: MIT
EOF

echo -e "${GREEN}✓${NC} 临时文件准备完成"

# 删除旧的 DMG
if [ -f "$DMG_PATH" ]; then
    echo -e "${BLUE}→${NC} 删除旧的 DMG..."
    rm "$DMG_PATH"
fi

# 创建临时 DMG
echo -e "${BLUE}→${NC} 创建临时 DMG..."
TMP_DMG="/tmp/${APP_NAME}_tmp.dmg"
rm -f "$TMP_DMG"

# 计算所需大小（应用大小 + 100MB 缓冲）
APP_SIZE=$(du -sm "$APP_PATH" | cut -f1)
DMG_SIZE=$((APP_SIZE + 100))

hdiutil create -srcfolder "$TMP_DMG_DIR" \
    -volname "$VOLUME_NAME" \
    -fs HFS+ \
    -fsargs "-c c=64,a=16,e=16" \
    -format UDRW \
    -size ${DMG_SIZE}m \
    "$TMP_DMG"

echo -e "${GREEN}✓${NC} 临时 DMG 创建完成"

# 挂载 DMG
echo -e "${BLUE}→${NC} 挂载 DMG 进行自定义..."
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "$TMP_DMG" | \
    egrep '^/dev/' | sed 1q | awk '{print $3}')

echo -e "${GREEN}✓${NC} DMG 已挂载到: $MOUNT_DIR"

# 使用 AppleScript 设置窗口布局
echo -e "${BLUE}→${NC} 设置窗口布局..."
sleep 2  # 等待挂载完成

osascript <<EOF
tell application "Finder"
    tell disk "$VOLUME_NAME"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set the bounds of container window to {100, 100, 700, 500}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 96
        
        -- 设置图标位置
        set position of item "${APP_NAME}.app" of container window to {150, 200}
        set position of item "Applications" of container window to {450, 200}
        
        -- 如果有 README，设置其位置
        if exists item "README.txt" then
            set position of item "README.txt" of container window to {300, 350}
        end if
        
        close
        open
        update without registering applications
        delay 2
    end tell
end tell
EOF

# 设置权限
echo -e "${BLUE}→${NC} 设置权限..."
chmod -Rf go-w "$MOUNT_DIR" 2>/dev/null || true
chmod -Rf a+r "$MOUNT_DIR" 2>/dev/null || true

# 同步并卸载
echo -e "${BLUE}→${NC} 同步文件系统..."
sync
sync
echo -e "${BLUE}→${NC} 卸载 DMG..."
hdiutil detach "$MOUNT_DIR" -quiet -force || true
sleep 5

echo -e "${GREEN}✓${NC} DMG 已卸载"

# 转换为压缩的只读格式
echo -e "${BLUE}→${NC} 压缩 DMG..."
hdiutil convert "$TMP_DMG" \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "$DMG_PATH"

echo -e "${GREEN}✓${NC} DMG 压缩完成"

# 清理临时文件
echo -e "${BLUE}→${NC} 清理临时文件..."
rm -f "$TMP_DMG"
rm -rf "$TMP_DMG_DIR"

# 显示结果
echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}  DMG 打包完成！${NC}"
echo -e "${GREEN}================================${NC}\n"
echo -e "文件位置: ${BLUE}$DMG_PATH${NC}"
echo -e "文件大小: ${BLUE}$(du -h "$DMG_PATH" | cut -f1)${NC}"
echo -e "\n${YELLOW}提示: 您可以测试 DMG 文件:${NC}"
echo -e "  open \"$DMG_PATH\"\n"

# 询问是否立即打开
read -p "$(echo -e ${YELLOW}是否立即打开 DMG 进行测试？ [y/N]: ${NC})" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    open "$DMG_PATH"
fi
