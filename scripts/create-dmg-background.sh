#!/bin/bash

# DMG 背景图生成脚本
# 使用 ImageMagick 创建自定义背景

set -e

echo "创建 DMG 背景图..."

# 检查 ImageMagick 是否安装
if ! command -v convert &> /dev/null; then
    echo "警告: 未找到 ImageMagick"
    echo "安装方法: brew install imagemagick"
    echo "跳过背景图创建..."
    exit 0
fi

# 创建背景图目录
BACKGROUND_DIR="dmg-assets"
mkdir -p "$BACKGROUND_DIR"

# 背景图尺寸
WIDTH=600
HEIGHT=400

# 创建背景图
convert -size ${WIDTH}x${HEIGHT} \
    gradient:#f0f0f0-#ffffff \
    -font Helvetica-Bold \
    -pointsize 20 \
    -fill '#0070f3' \
    -gravity North \
    -annotate +0+50 '代理订阅管理器' \
    -pointsize 14 \
    -fill '#666666' \
    -annotate +0+80 'Proxy Subscription Manager' \
    -pointsize 12 \
    -annotate +0+250 '← 将应用拖到 Applications 文件夹' \
    "$BACKGROUND_DIR/background.png"

echo "✓ 背景图创建完成: $BACKGROUND_DIR/background.png"
