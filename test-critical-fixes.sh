#!/bin/bash

echo "🧪 测试关键修复"
echo "================"
echo ""

# 清理
echo "1️⃣ 清理进程..."
pkill -f "tauri" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true
sleep 2

# 编译检查
echo ""
echo "2️⃣ TypeScript 检查..."
bun run tsc --noEmit
if [ $? -ne 0 ]; then
    echo "❌ TypeScript 检查失败"
    exit 1
fi
echo "✓ TypeScript OK"

echo ""
echo "3️⃣ Rust 检查..."
cargo check --manifest-path src-tauri/Cargo.toml --quiet
if [ $? -ne 0 ]; then
    echo "❌ Rust 检查失败"
    exit 1
fi
echo "✓ Rust OK"

echo ""
echo "4️⃣ 运行测试..."
cargo test --manifest-path src-tauri/Cargo.toml --quiet
if [ $? -ne 0 ]; then
    echo "❌ 测试失败"
    exit 1
fi
echo "✓ 测试通过"

echo ""
echo "✅ 所有检查通过！"
echo ""
echo "📋 关键修复内容:"
echo "  ✓ 移除导致崩溃的 ctrl_c 信号处理"
echo "  ✓ 统一配置文件路径获取 (get_config_path)"
echo "  ✓ 所有 Tauri command 添加 app_handle 参数"
echo "  ✓ 自动创建应用数据目录和默认配置"
echo "  ✓ 支持开发模式和生产模式"
echo ""
echo "🚀 现在可以启动应用测试："
echo "   bun run dev"
echo ""
echo "⚠️  请测试以下功能："
echo "   1. 添加订阅"
echo "   2. 编辑订阅"  
echo "   3. 删除订阅 ⭐ (之前不工作)"
echo "   4. 启动服务器 ⭐ (之前崩溃)"
echo "   5. 访问 http://127.0.0.1:8080/config"
echo "   6. 停止服务器"
echo ""
