#!/bin/bash

# E2E 测试脚本 - 代理订阅管理器
# 测试完整的工作流程

set -e

echo "🧪 开始端到端测试"
echo "===================="

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试结果计数
TESTS_PASSED=0
TESTS_FAILED=0

# 测试函数
test_case() {
    echo -e "\n${YELLOW}测试:${NC} $1"
}

pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((TESTS_PASSED++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((TESTS_FAILED++))
}

# 1. 检查配置文件
test_case "检查配置文件是否存在"
if [ -f "subscriptions.json" ]; then
    pass "subscriptions.json 存在"
else
    fail "subscriptions.json 不存在"
fi

if [ -f "_docs/basic.yml" ]; then
    pass "_docs/basic.yml 存在"
else
    fail "_docs/basic.yml 不存在"
fi

# 2. 验证配置文件格式
test_case "验证 JSON 配置文件格式"
if node -e "JSON.parse(require('fs').readFileSync('subscriptions.json', 'utf8'))" 2>/dev/null; then
    pass "subscriptions.json 是有效的 JSON"
else
    fail "subscriptions.json JSON 格式无效"
fi

# 3. 检查构建产物
test_case "检查前端构建产物"
if [ -d "dist" ]; then
    pass "dist 目录存在"
    
    if [ -f "dist/index.html" ]; then
        pass "dist/index.html 存在"
    else
        fail "dist/index.html 不存在"
    fi
else
    fail "dist 目录不存在"
fi

# 4. 检查 Rust 编译产物
test_case "检查 Rust 编译产物"
if [ -f "src-tauri/target/release/proxy-sub-manager" ] || [ -f "src-tauri/target/release/proxy-sub-manager.exe" ]; then
    pass "Rust 可执行文件存在"
else
    fail "Rust 可执行文件不存在（可能需要先运行 'cargo build --release'）"
fi

# 5. 运行 Rust 单元测试
test_case "运行 Rust 单元测试"
if cargo test --manifest-path src-tauri/Cargo.toml --quiet 2>&1 | grep -q "test result: ok"; then
    pass "Rust 单元测试通过"
else
    fail "Rust 单元测试失败"
fi

# 6. 检查 TypeScript 类型
test_case "检查 TypeScript 类型"
if npx tsc --noEmit 2>&1; then
    pass "TypeScript 类型检查通过"
else
    fail "TypeScript 类型检查失败"
fi

# 7. 测试 HTTP 服务器端点（如果服务器正在运行）
test_case "测试 HTTP 服务器端点（可选）"
if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
    pass "服务器 /health 端点响应正常"
    
    if curl -s http://127.0.0.1:8080/config > /dev/null 2>&1; then
        pass "服务器 /config 端点响应正常"
    else
        fail "服务器 /config 端点无响应"
    fi
else
    echo -e "${YELLOW}⚠ SKIP${NC}: 服务器未运行（这是正常的）"
fi

# 8. 检查代码格式（Rust）
test_case "检查 Rust 代码格式"
if cargo fmt --manifest-path src-tauri/Cargo.toml --check 2>&1 | grep -q "Diff"; then
    fail "Rust 代码需要格式化（运行 'cargo fmt'）"
else
    pass "Rust 代码格式正确"
fi

# 9. 运行 Clippy 检查
test_case "运行 Clippy 检查"
if cargo clippy --manifest-path src-tauri/Cargo.toml --quiet 2>&1 | grep -q "warning"; then
    echo -e "${YELLOW}⚠ WARN${NC}: Clippy 发现一些警告"
else
    pass "Clippy 检查通过"
fi

# 总结
echo ""
echo "===================="
echo "📊 测试总结"
echo "===================="
echo -e "${GREEN}通过:${NC} $TESTS_PASSED"
echo -e "${RED}失败:${NC} $TESTS_FAILED"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 所有测试通过！${NC}"
    exit 0
else
    echo -e "\n${RED}❌ 有 $TESTS_FAILED 个测试失败${NC}"
    exit 1
fi
