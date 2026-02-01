#!/bin/bash

echo "=========================================="
echo "🔍 调试模式测试"
echo "=========================================="
echo ""
echo "📝 测试步骤："
echo "1. 应用启动后，打开开发者工具（DevTools）"
echo "2. 点击「删除」按钮，观察控制台输出"
echo "3. 点击「启动服务器」按钮，观察控制台输出"
echo ""
echo "💡 提示："
echo "- 前端日志在浏览器控制台（Console）"
echo "- 后端日志在终端窗口"
echo ""
echo "按 Ctrl+C 停止应用"
echo ""
echo "=========================================="
echo ""

cd /Users/lei/dev/personal/proxy-sub-manager
bun run tauri dev
