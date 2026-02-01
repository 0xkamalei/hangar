#!/bin/bash

# 开发模式测试脚本
# 启动应用并进行基本测试

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  开发模式测试  ${NC}"
echo -e "${BLUE}================================${NC}\n"

# 1. 清理旧进程
echo -e "${YELLOW}[1/4]${NC} 清理旧进程..."
pkill -f "hangar" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓${NC} 清理完成\n"

# 2. 检查配置文件
echo -e "${YELLOW}[2/4]${NC} 检查配置文件..."
if [ ! -f "subscriptions.json" ]; then
    echo -e "${RED}✗${NC} 缺少 subscriptions.json"
    exit 1
fi
if [ ! -f "basic_test.yml" ] && [ ! -f "_docs/basic.yml" ]; then
    echo -e "${RED}✗${NC} 缺少基础配置文件"
    exit 1
fi
echo -e "${GREEN}✓${NC} 配置文件完整\n"

# 3. 启动开发服务器
echo -e "${YELLOW}[3/4]${NC} 启动开发服务器..."
echo -e "${BLUE}→${NC} 运行: bun run tauri:dev"
echo -e "${YELLOW}提示: 按 Ctrl+C 停止${NC}\n"

# 在后台启动，但保持输出
bun run tauri:dev &
DEV_PID=$!

# 等待启动
sleep 5

# 4. 检查是否启动成功
echo -e "\n${YELLOW}[4/4]${NC} 检查应用状态..."
if ps -p $DEV_PID > /dev/null; then
    echo -e "${GREEN}✓${NC} 应用正在运行 (PID: $DEV_PID)\n"
    
    echo -e "${GREEN}================================${NC}"
    echo -e "${GREEN}  开发模式已启动！${NC}"
    echo -e "${GREEN}================================${NC}\n"
    echo -e "测试清单："
    echo -e "  [ ] 应用窗口正常显示"
    echo -e "  [ ] 可以添加订阅"
    echo -e "  [ ] 可以启动服务器"
    echo -e "  [ ] 服务器状态指示灯正常"
    echo -e "  [ ] 订阅列表显示正常"
    echo -e ""
    echo -e "${YELLOW}按 Ctrl+C 停止开发服务器${NC}\n"
    
    # 等待用户中断
    wait $DEV_PID
else
    echo -e "${RED}✗${NC} 应用启动失败\n"
    exit 1
fi
