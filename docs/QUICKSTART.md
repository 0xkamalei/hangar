# Hangar 快速开始指南

## 10 分钟上手 Hangar

### 第一步：安装 (2 分钟)

```bash
# 克隆仓库
git clone <your-repo-url>
cd hangar

# 一键安装
./install.sh
```

等待构建完成，`hangar` 命令将全局可用。

### 第二步：添加订阅 (1 分钟)

```bash
# 添加你的 Clash 订阅
hangar sub add "https://your-subscription-url" --name "我的订阅"

# 查看已添加的订阅
hangar sub list
```

### 第三步：生成配置 (1 分钟)

```bash
# 合并所有订阅和规则
hangar merge
```

这会在 `~/.hangar/current.yaml` 生成最终配置文件。

### 第四步：启动服务器 (1 分钟)

```bash
# 方式一：使用管理脚本（推荐）
./hangar-server.sh start --port 8080 --interval 3600

# 方式二：直接运行
hangar serve --daemon --port 8080 --interval 3600
```

参数说明：
- `--port 8080`: 监听 8080 端口
- `--interval 3600`: 每小时自动更新订阅
- `--daemon`: 后台运行

### 第五步：在 Clash 中使用 (2 分钟)

1. 打开你的 Clash 客户端（Clash for Windows / ClashX / Clash Verge 等）
2. 添加配置订阅：
   ```
   http://127.0.0.1:8080/config
   ```
3. 更新订阅
4. 选择节点并启用代理

### 第六步：验证服务 (1 分钟)

```bash
# 查看服务状态
./hangar-server.sh status

# 查看日志
./hangar-server.sh logs

# 测试配置 URL
curl http://127.0.0.1:8080/config | head -20
```

### 第七步：日常管理 (2 分钟)

```bash
# 查看实时日志
./hangar-server.sh logs -f

# 添加新订阅
hangar sub add "https://another-url" --name "订阅2"
hangar merge  # 重新合并配置（会自动被服务器检测并重载）

# 重启服务
./hangar-server.sh restart

# 停止服务
./hangar-server.sh stop
```

## 进阶功能

### 使用 AI 辅助配置（需要配置 LLM）

```bash
# 配置 OpenAI API
hangar config --api-key "sk-xxx" --base-url "https://api.openai.com/v1"

# 使用自然语言修改配置
hangar ai "添加一个香港节点组"
hangar ai "将所有美国节点设为自动选择"
```

### 查看历史版本

```bash
# 查看所有配置版本
hangar history list

# 回滚到某个版本
hangar history rollback <version-id>

# 对比两个历史版本
hangar history diff <v1> <v2>

# 对比历史版本和当前配置（默认 current.yaml，兼容 current.yml）
hangar history diff <v1>
```

### 手动编辑配置

```bash
# 用默认编辑器打开配置
hangar editor

# 编辑后保存，服务器会自动检测并重载
```

## 常见场景

### 场景 1：更换订阅

```bash
# 删除旧订阅
hangar sub list  # 查看订阅 ID
hangar sub remove <old-id>

# 添加新订阅
hangar sub add "https://new-url" --name "新订阅"

# 重新生成配置
hangar merge
```

服务器会自动检测到配置变化并重载，Clash 客户端会在下次更新时拉取新配置。

### 场景 2：临时停止自动更新

```bash
# 停止当前服务
./hangar-server.sh stop

# 重启，但不启用自动更新（interval=0）
./hangar-server.sh start --port 8080
```

### 场景 3：多个订阅合并

```bash
# 添加多个订阅
hangar sub add "https://url1" --name "订阅1"
hangar sub add "https://url2" --name "订阅2"
hangar sub add "https://url3" --name "订阅3"

# 一次性合并所有订阅
hangar merge

# Hangar 会自动：
# 1. 按地区分组节点
# 2. 去重
# 3. 生成代理组
# 4. 合并规则
```

### 场景 3.5：临时禁用/启用订阅

```bash
# 查看所有订阅及其状态
hangar sub list

# 临时禁用某个订阅（不删除）
hangar sub disable 0  # 使用索引
# 或
hangar sub disable a1b2c3d4-5678-90ab-cdef-1234567890ab  # 使用 ID

# 重新生成配置（禁用的订阅会被跳过）
hangar merge

# 需要时重新启用
hangar sub enable 0

# 💡 提示：禁用的订阅不会被删除，也不会参与自动更新和合并
# 详细说明请查看 SUBSCRIPTION_ENABLE_DISABLE.md
```


### 场景 4：远程访问（VPS 部署）

```bash
# 在 VPS 上启动，监听所有接口
hangar serve --daemon --host 0.0.0.0 --port 8080 --interval 3600

# 在本地 Clash 中使用
# http://<VPS-IP>:8080/config

# ⚠️ 注意：建议配置防火墙或使用 nginx 反向代理
```

## 故障排查速查

| 问题 | 解决方法 |
|------|----------|
| 命令未找到 | 运行 `./install.sh` 重新安装 |
| 端口被占用 | 使用 `--port` 指定其他端口 |
| 订阅下载失败 | 检查网络，查看日志 `./hangar-server.sh logs` |
| 配置未更新 | 手动 `hangar merge`，然后 `./hangar-server.sh restart` |
| 服务启动失败 | 查看日志 `cat ~/.hangar/server.log` |

## 卸载

```bash
# 停止服务
./hangar-server.sh stop

# 删除二进制文件
sudo rm /usr/local/bin/hangar

# 删除配置（可选）
rm -rf ~/.hangar
```

## 下一步

- 📖 查看 [完整文档](README.md)
- 🔧 了解 [CLI 命令详解](cli.md)
- 🚀 阅读 [Daemon 模式说明](DAEMON_UPDATE_SUMMARY.md)
- 👀 学习 [文件监控功能](FILE_WATCH_FEATURE.md)

祝使用愉快！🎉
