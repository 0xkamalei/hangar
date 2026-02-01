#!/bin/bash

# 验证脚本 - 确保所有配置正确

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  项目验证脚本  ${NC}"
echo -e "${BLUE}================================${NC}\n"

PASSED=0
FAILED=0

test_pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

test_fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

test_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. 检查依赖
echo -e "${YELLOW}[1/10]${NC} 检查依赖..."
if command -v bun &> /dev/null; then
    test_pass "Bun 已安装 ($(bun --version))"
else
    test_fail "Bun 未安装"
fi

if command -v cargo &> /dev/null; then
    test_pass "Cargo 已安装 ($(cargo --version | head -1))"
else
    test_fail "Cargo 未安装"
fi

if command -v node &> /dev/null; then
    test_pass "Node.js 已安装 ($(node --version))"
else
    test_warn "Node.js 未安装（可选）"
fi

# 2. 检查配置文件
echo -e "\n${YELLOW}[2/10]${NC} 检查配置文件..."
if [ -f "subscriptions.json" ]; then
    if node -e "JSON.parse(require('fs').readFileSync('subscriptions.json', 'utf8'))" 2>/dev/null || bun run -e "JSON.parse(require('fs').readFileSync('subscriptions.json', 'utf8'))" 2>/dev/null; then
        test_pass "subscriptions.json 格式正确"
    else
        test_fail "subscriptions.json 格式错误"
    fi
else
    test_fail "subscriptions.json 不存在"
fi

if [ -f "basic_test.yml" ] || [ -f "_docs/basic.yml" ]; then
    test_pass "基础配置文件存在"
else
    test_fail "基础配置文件不存在"
fi

if [ -f "src-tauri/tauri.conf.json" ]; then
    test_pass "tauri.conf.json 存在"
else
    test_fail "tauri.conf.json 不存在"
fi

# 3. 检查 Cargo 配置
echo -e "\n${YELLOW}[3/10]${NC} 检查 Cargo 配置..."
if grep -q "default-run" src-tauri/Cargo.toml; then
    test_pass "Cargo.toml 包含 default-run"
else
    test_fail "Cargo.toml 缺少 default-run"
fi

# 4. TypeScript 类型检查
echo -e "\n${YELLOW}[4/10]${NC} TypeScript 类型检查..."
if bun run tsc --noEmit 2>&1 | grep -q "error"; then
    test_fail "TypeScript 类型检查失败"
else
    test_pass "TypeScript 类型检查通过"
fi

# 5. Rust 编译检查
echo -e "\n${YELLOW}[5/10]${NC} Rust 编译检查..."
if cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | grep -q "error"; then
    test_fail "Rust 编译检查失败"
else
    test_pass "Rust 编译检查通过"
fi

# 6. Rust 测试
echo -e "\n${YELLOW}[6/10]${NC} Rust 单元测试..."
if cargo test --manifest-path src-tauri/Cargo.toml --quiet 2>&1 | grep -q "test result: ok"; then
    test_pass "Rust 单元测试通过"
else
    test_fail "Rust 单元测试失败"
fi

# 7. Clippy 检查
echo -e "\n${YELLOW}[7/10]${NC} Clippy 代码检查..."
CLIPPY_OUTPUT=$(cargo clippy --manifest-path src-tauri/Cargo.toml 2>&1)
if echo "$CLIPPY_OUTPUT" | grep -q "error:"; then
    test_fail "Clippy 发现错误"
elif echo "$CLIPPY_OUTPUT" | grep -q "warning:"; then
    test_warn "Clippy 发现警告"
else
    test_pass "Clippy 检查通过"
fi

# 8. 检查图标文件
echo -e "\n${YELLOW}[8/10]${NC} 检查图标文件..."
ICON_FILES=("src-tauri/icons/32x32.png" "src-tauri/icons/128x128.png" "src-tauri/icons/icon.icns")
ICON_OK=true
for icon in "${ICON_FILES[@]}"; do
    if [ ! -f "$icon" ]; then
        ICON_OK=false
        break
    fi
done

if $ICON_OK; then
    test_pass "所有图标文件存在"
else
    test_fail "缺少图标文件"
fi

# 9. 检查脚本权限
echo -e "\n${YELLOW}[9/10]${NC} 检查脚本权限..."
SCRIPTS=("scripts/simple-dmg.sh" "scripts/test-dev.sh" "scripts/verify.sh")
SCRIPTS_OK=true
for script in "${SCRIPTS[@]}"; do
    if [ ! -x "$script" ]; then
        SCRIPTS_OK=false
        break
    fi
done

if $SCRIPTS_OK; then
    test_pass "所有脚本有执行权限"
else
    test_warn "部分脚本缺少执行权限"
    echo "    运行: chmod +x scripts/*.sh"
fi

# 10. 检查 package.json 脚本
echo -e "\n${YELLOW}[10/10]${NC} 检查 package.json 脚本..."
if grep -q '"dev": "tauri dev"' package.json; then
    test_pass "package.json dev 脚本配置正确"
else
    test_fail "package.json dev 脚本配置错误"
fi

if grep -q '"dev:frontend": "vite"' package.json; then
    test_pass "package.json dev:frontend 脚本配置正确"
else
    test_fail "package.json dev:frontend 脚本配置错误"
fi

# 总结
echo -e "\n${BLUE}================================${NC}"
echo -e "${BLUE}  验证结果  ${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "${GREEN}通过: $PASSED${NC}"
echo -e "${RED}失败: $FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 所有检查通过！${NC}"
    echo -e "\n下一步:"
    echo -e "  ${BLUE}→${NC} 运行开发模式: ${YELLOW}bun run dev${NC}"
    echo -e "  ${BLUE}→${NC} 测试命令行工具: ${YELLOW}cargo run --bin cli -- subs.txt test.yml${NC}"
    echo -e "  ${BLUE}→${NC} 构建应用: ${YELLOW}bun run build:all${NC}"
    exit 0
else
    echo -e "\n${RED}❌ 有 $FAILED 个检查失败${NC}"
    echo -e "\n请修复上述问题后重新运行验证"
    exit 1
fi
