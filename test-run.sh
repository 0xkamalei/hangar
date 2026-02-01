#!/bin/bash

echo "🧪 测试 Proxy Subscription Manager"
echo "=================================="
echo ""

# 清理之前的进程
echo "1️⃣ 清理之前的进程..."
pkill -f "bun run dev" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true
pkill -f "tauri" 2>/dev/null || true
pkill -f "proxy-sub-manager" 2>/dev/null || true
sleep 2

# 检查必要的文件
echo ""
echo "2️⃣ 检查必要的文件..."
if [ ! -f "subscriptions.json" ]; then
    echo "❌ subscriptions.json 不存在"
    exit 1
fi
echo "✓ subscriptions.json 存在"

if [ ! -f "basic_test.yml" ]; then
    echo "❌ basic_test.yml 不存在"
    exit 1
fi
echo "✓ basic_test.yml 存在"

# 运行 TypeScript 类型检查
echo ""
echo "3️⃣ 运行 TypeScript 类型检查..."
bun run tsc --noEmit
if [ $? -ne 0 ]; then
    echo "❌ TypeScript 类型检查失败"
    exit 1
fi
echo "✓ TypeScript 类型检查通过"

# 运行 Rust 测试
echo ""
echo "4️⃣ 运行 Rust 测试..."
cargo test --manifest-path src-tauri/Cargo.toml --quiet
if [ $? -ne 0 ]; then
    echo "❌ Rust 测试失败"
    exit 1
fi
echo "✓ Rust 测试通过"

# 启动开发服务器
echo ""
echo "5️⃣ 启动开发服务器..."
echo ""
echo "⚠️  请手动测试以下功能："
echo "   1. 添加订阅（无 placeholder）"
echo "   2. 编辑订阅"
echo "   3. 删除订阅"
echo "   4. 启用/禁用订阅"
echo "   5. 启动服务器（检查是否崩溃）"
echo "   6. 访问 http://127.0.0.1:8080/config"
echo "   7. 停止服务器"
echo "   8. 检查成功/错误消息是否可见且3秒后消失"
echo ""
echo "🚀 正在启动应用..."
echo ""

bun run dev
