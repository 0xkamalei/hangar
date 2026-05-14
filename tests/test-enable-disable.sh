#!/bin/bash

# 测试 enable/disable 命令的脚本

echo "🧪 Testing hangar sub enable/disable commands"
echo "=============================================="
echo ""

# 构建项目
echo "📦 Building hangar..."
cd src-tauri
cargo build --release 2>&1 | tail -5
cd ..
echo ""

# 使用 release 二进制
HANGAR="./src-tauri/target/release/hangar"

# 测试 1: 查看当前订阅列表
echo "✅ Test 1: List subscriptions"
$HANGAR sub list
echo ""

# 测试 2: 如果没有订阅，添加一个测试订阅
echo "✅ Test 2: Checking if we need to add test subscription"
SUB_COUNT=$($HANGAR sub list 2>/dev/null | grep -v "ID" | grep -v "No subscriptions" | wc -l)
if [ "$SUB_COUNT" -eq 0 ]; then
    echo "No subscriptions found, adding a test one..."
    $HANGAR sub add "https://example.com/test" --name "测试订阅"
    echo ""
fi

# 测试 3: 禁用第一个订阅
echo "✅ Test 3: Disable subscription at index 0"
$HANGAR sub disable 0
echo ""

# 测试 4: 再次列出订阅，验证状态
echo "✅ Test 4: Verify disabled status"
$HANGAR sub list
echo ""

# 测试 5: 重新启用订阅
echo "✅ Test 5: Re-enable subscription at index 0"
$HANGAR sub enable 0
echo ""

# 测试 6: 最终验证
echo "✅ Test 6: Final verification"
$HANGAR sub list
echo ""

echo "🎉 All tests completed!"
echo ""
echo "📝 Notes:"
echo "   - Subscriptions can be enabled/disabled by index or UUID"
echo "   - Disabled subscriptions will be skipped in merge and auto-updates"
echo "   - See SUBSCRIPTION_ENABLE_DISABLE.md for detailed usage guide"
