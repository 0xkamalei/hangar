#!/bin/bash

# 应用图标生成脚本
# 使用 ImageMagick 创建简单但专业的图标

set -e

echo "🎨 生成应用图标..."

# 检查 ImageMagick
if ! command -v convert &> /dev/null; then
    echo "⚠️  需要安装 ImageMagick"
    echo "运行: brew install imagemagick"
    exit 1
fi

ICON_DIR="src-tauri/icons"
mkdir -p "$ICON_DIR"

# 创建基础 1024x1024 图标
convert -size 1024x1024 \
    -define gradient:angle=135 \
    gradient:'#0070f3'-'#00d4ff' \
    -gravity center \
    \( -size 800x800 xc:none \
       -fill white \
       -draw "roundrectangle 0,0 800,800 100,100" \
    \) \
    -composite \
    -gravity center \
    -font Helvetica-Bold \
    -pointsize 280 \
    -fill '#0070f3' \
    -annotate +0-50 '🔗' \
    -pointsize 120 \
    -fill white \
    -annotate +0+180 'PSM' \
    "$ICON_DIR/icon_1024x1024.png"

echo "✓ 1024x1024 图标创建完成"

# 生成不同尺寸
for size in 512 256 128 32; do
    convert "$ICON_DIR/icon_1024x1024.png" \
        -resize ${size}x${size} \
        "$ICON_DIR/${size}x${size}.png"
    echo "✓ ${size}x${size} 图标创建完成"
    
    # 创建 @2x 版本
    if [ $size -eq 128 ]; then
        cp "$ICON_DIR/256x256.png" "$ICON_DIR/128x128@2x.png"
        echo "✓ 128x128@2x 图标创建完成"
    fi
done

# 创建 .icns 文件 (macOS)
echo "→ 创建 .icns 文件..."
ICONSET_DIR="$ICON_DIR/icon.iconset"
mkdir -p "$ICONSET_DIR"

# 复制到 iconset 目录
cp "$ICON_DIR/32x32.png" "$ICONSET_DIR/icon_16x16@2x.png"
cp "$ICON_DIR/32x32.png" "$ICONSET_DIR/icon_32x32.png"
cp "$ICON_DIR/128x128.png" "$ICONSET_DIR/icon_64x64@2x.png"
cp "$ICON_DIR/128x128.png" "$ICONSET_DIR/icon_128x128.png"
cp "$ICON_DIR/256x256.png" "$ICONSET_DIR/icon_128x128@2x.png"
cp "$ICON_DIR/256x256.png" "$ICONSET_DIR/icon_256x256.png"
cp "$ICON_DIR/512x512.png" "$ICONSET_DIR/icon_256x256@2x.png"
cp "$ICON_DIR/512x512.png" "$ICONSET_DIR/icon_512x512.png"
cp "$ICON_DIR/icon_1024x1024.png" "$ICONSET_DIR/icon_512x512@2x.png"

# 生成 .icns
iconutil -c icns "$ICONSET_DIR" -o "$ICON_DIR/icon.icns"
rm -rf "$ICONSET_DIR"
echo "✓ icon.icns 创建完成"

# 创建 .ico 文件 (Windows)
if command -v icotool &> /dev/null; then
    icotool -c -o "$ICON_DIR/icon.ico" \
        "$ICON_DIR/32x32.png" \
        "$ICON_DIR/128x128.png" \
        "$ICON_DIR/256x256.png"
    echo "✓ icon.ico 创建完成"
else
    echo "⚠️  跳过 .ico 创建 (需要 icotool)"
fi

echo ""
echo "🎉 图标生成完成！"
echo "图标位置: $ICON_DIR/"
ls -lh "$ICON_DIR/"/*.{png,icns,ico} 2>/dev/null || ls -lh "$ICON_DIR/"/*.{png,icns}
