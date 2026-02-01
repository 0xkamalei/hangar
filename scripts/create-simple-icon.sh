#!/bin/bash

# 使用 sips 和系统工具创建简单图标

set -e

echo "🎨 创建简单应用图标..."

ICON_DIR="src-tauri/icons"
mkdir -p "$ICON_DIR"

# 使用 SF Symbols 的网络图标创建基础图标
# 如果没有图标文件，我们创建一个纯色占位符

# 创建一个简单的纯色图标作为临时方案
# 蓝色渐变背景，代表网络/代理

cat > "$ICON_DIR/create_icon.py" << 'PYTHON_SCRIPT'
#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont
import os

# 创建 1024x1024 图标
size = 1024
img = Image.new('RGB', (size, size), color='#0070f3')

# 添加圆角
draw = ImageDraw.Draw(img)

# 画一个白色圆角矩形
margin = 100
rect_size = size - 2 * margin
draw.rounded_rectangle(
    [(margin, margin), (margin + rect_size, margin + rect_size)],
    radius=100,
    fill='white'
)

# 添加文字
try:
    # 尝试使用系统字体
    font_large = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 280)
    font_small = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', 120)
except:
    font_large = ImageFont.load_default()
    font_small = ImageFont.load_default()

# 画 "🔗" emoji (如果支持) 或 "P"
text_top = "P"
text_bottom = "SM"

# 绘制主图标文字
draw.text((size//2, size//2 - 100), text_top, 
          fill='#0070f3', font=font_large, anchor='mm')
draw.text((size//2, size//2 + 150), text_bottom, 
          fill='#0070f3', font=font_small, anchor='mm')

# 保存 PNG
icon_dir = os.path.dirname(os.path.abspath(__file__))
icon_path = os.path.join(icon_dir, 'icon_1024x1024.png')
img.save(icon_path, 'PNG')
print(f"✓ 创建基础图标: {icon_path}")

# 创建不同尺寸
for size in [512, 256, 128, 32]:
    resized = img.resize((size, size), Image.Resampling.LANCZOS)
    path = os.path.join(icon_dir, f'{size}x{size}.png')
    resized.save(path, 'PNG')
    print(f"✓ 创建 {size}x{size} 图标")
    
    # 创建 @2x 版本
    if size == 128:
        img256 = img.resize((256, 256), Image.Resampling.LANCZOS)
        path_2x = os.path.join(icon_dir, '128x128@2x.png')
        img256.save(path_2x, 'PNG')
        print(f"✓ 创建 128x128@2x 图标")

print("✓ 所有图标创建完成")
PYTHON_SCRIPT

# 运行 Python 脚本
if command -v python3 &> /dev/null; then
    chmod +x "$ICON_DIR/create_icon.py"
    cd "$ICON_DIR" && python3 create_icon.py
    rm create_icon.py
    
    # 创建 .icns
    if [ -f "icon_1024x1024.png" ]; then
        ICONSET_DIR="icon.iconset"
        mkdir -p "$ICONSET_DIR"
        
        cp 32x32.png "$ICONSET_DIR/icon_16x16@2x.png"
        cp 32x32.png "$ICONSET_DIR/icon_32x32.png"
        cp 128x128.png "$ICONSET_DIR/icon_64x64@2x.png"
        cp 128x128.png "$ICONSET_DIR/icon_128x128.png"
        cp 256x256.png "$ICONSET_DIR/icon_128x128@2x.png"
        cp 256x256.png "$ICONSET_DIR/icon_256x256.png"
        cp 512x512.png "$ICONSET_DIR/icon_256x256@2x.png"
        cp 512x512.png "$ICONSET_DIR/icon_512x512.png"
        cp icon_1024x1024.png "$ICONSET_DIR/icon_512x512@2x.png"
        
        iconutil -c icns "$ICONSET_DIR" -o icon.icns
        rm -rf "$ICONSET_DIR"
        echo "✓ icon.icns 创建完成"
    fi
    
    cd - > /dev/null
    
    echo ""
    echo "🎉 图标创建完成！"
    ls -lh "$ICON_DIR"/*.{png,icns} 2>/dev/null | head -10
else
    echo "❌ 需要 Python 3"
    exit 1
fi
