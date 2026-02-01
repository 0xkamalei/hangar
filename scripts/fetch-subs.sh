#!/bin/bash

# 获取订阅内容并分析代理名称

set -e

echo "📡 获取订阅内容..."

# 临时目录
TMP_DIR="/tmp/proxy-sub-test"
mkdir -p "$TMP_DIR"

# 读取 subs.txt
while IFS= read -r url; do
    # 跳过空行和注释
    [[ -z "$url" || "$url" =~ ^# ]] && continue
    
    # 生成文件名
    filename=$(echo "$url" | md5 | head -c 8)
    output="$TMP_DIR/sub_${filename}.yaml"
    
    echo ""
    echo "→ 获取订阅: $url"
    
    # 下载订阅
    if curl -s "$url" -o "$output.tmp"; then
        # 尝试 base64 解码
        if base64 -d "$output.tmp" > "$output" 2>/dev/null; then
            echo "  ✓ Base64 解码成功"
        else
            mv "$output.tmp" "$output"
            echo "  ✓ 直接保存"
        fi
        
        # 提取代理名称
        echo "  → 代理列表:"
        grep -E "^\s*-\s*name:" "$output" | head -20 | sed 's/.*name: /    /'
        
        proxy_count=$(grep -c "^\s*-\s*name:" "$output" || echo "0")
        echo "  ✓ 共 $proxy_count 个代理"
    else
        echo "  ✗ 获取失败"
    fi
done < "subs.txt"

echo ""
echo "✓ 订阅内容已保存到: $TMP_DIR/"
echo ""
echo "分析代理名称模式..."
echo ""

# 合并所有代理名称
cat "$TMP_DIR"/sub_*.yaml 2>/dev/null | grep -E "^\s*-\s*name:" | sed 's/.*name: //' | sort > "$TMP_DIR/all_names.txt"

# 分析地区模式
echo "📊 地区分布:"
for region in "香港" "HK" "台湾" "TW" "日本" "JP" "新加坡" "SG" "美国" "US" "英国" "UK"; do
    count=$(grep -ci "$region" "$TMP_DIR/all_names.txt" || echo "0")
    if [ "$count" -gt 0 ]; then
        echo "  $region: $count 个节点"
        grep -i "$region" "$TMP_DIR/all_names.txt" | head -3 | sed 's/^/    示例: /'
    fi
done

echo ""
echo "💡 建议："
echo "  - 检查地区识别是否准确"
echo "  - 确认特殊服务组的地区选择"
echo "  - 查看完整列表: cat $TMP_DIR/all_names.txt"
