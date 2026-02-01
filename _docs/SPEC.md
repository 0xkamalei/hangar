# Hangar - Product Specification

> AI-Powered Clash Config Manager

## 1. 产品概述

Hangar 是一个面向 C 端用户的桌面应用，用于管理代理订阅（机场）、合并配置、并通过 AI 智能修改 Clash 规则。

### 1.1 目标用户
- 使用多个机场订阅的用户
- 需要自定义 Clash 规则但不熟悉 YAML 语法的用户
- 希望快速配置特定服务代理策略的用户

### 1.2 核心价值
- **聚合**：多机场订阅一键合并
- **智能**：用自然语言描述需求，AI 自动生成配置
- **可控**：完整的版本历史，随时回退

---

## 2. 功能架构

### 2.1 UI 结构 (三个 Tab)

```
┌─────────────────────────────────────────────────────────┐
│  [仪表盘]  [机场管理]  [设置]                              │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Tab 内容区域                                            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

#### Tab 1: Main (主界面)

**功能区块：**

1. **AI 指令输入区**
   - 文本输入框，支持多行
   - 示例 placeholder: "让 Google 走台湾节点"
   - 提交按钮，触发 AI 生成

2. **配置预览区**
   - 显示当前生效的配置摘要
   - 显示 AI 修改的 diff（高亮增删改）
   - 确认/拒绝 AI 修改

3. **版本历史面板**
   - 版本列表（时间戳 + 简短描述）
   - 点击版本查看 diff
   - 回退到指定版本
   - 删除历史版本

4. **服务器控制**
   - 启动/停止按钮
   - 服务状态指示
   - 订阅地址显示 (http://127.0.0.1:PORT/config)
   - 一键复制订阅链接

5. **手动编辑**
   - "在编辑器中打开" 按钮
   - 调用系统默认编辑器打开配置文件
   - 文件修改后自动检测并提示重载

#### Tab 2: 机场管理

**功能区块：**

1. **订阅列表**
   - 显示所有已添加的机场
   - 每行：名称、URL（脱敏）、状态、节点数、最后更新时间
   - 启用/禁用开关
   - 编辑/删除按钮

2. **添加订阅**
   - 输入框：名称、订阅 URL
   - 添加按钮
   - 支持批量导入（多行 URL）

3. **操作按钮**
   - 刷新全部订阅
   - 导出/导入订阅列表

#### Tab 3: 设置

**功能区块：**

1. **LLM 配置**
   - API Base URL（默认 https://api.openai.com/v1）
   - API Key（密码输入框）
   - Model 选择（下拉或输入，默认 gpt-4o）
   - 测试连接按钮

2. **服务器配置**
   - 端口号（默认 8080）
   - 绑定地址（默认 127.0.0.1）

3. **规则库管理**
   - 内置规则列表（显示已包含的规则集）
   - 在线规则订阅 URL 列表
   - 添加/删除订阅规则源
   - 刷新规则按钮

4. **语言设置**
   - 语言切换（中文/English）
   - 跟随系统语言选项

5. **其他设置**
   - 数据目录位置
   - 重置应用数据

> **注**: License 系统暂不实现，后续版本添加。

---

## 3. 数据架构

### 3.1 文件存储结构

```
~/.hangar/                          # 应用数据目录
├── config.json                     # 应用配置
├── subscriptions.json              # 机场订阅列表
├── cache/                          # 缓存目录
│   ├── proxies/                    # 各机场解析出的节点缓存
│   │   ├── {subscription_id}.json
│   │   └── ...
│   └── rules/                      # 规则缓存
│       ├── builtin/                # 内置规则
│       └── remote/                 # 远程规则缓存
├── versions/                       # 版本快照
│   ├── v_1706784000_initial.yaml
│   ├── v_1706784100_ai_google_tw.yaml
│   └── ...
├── current.yaml                    # 当前生效的配置
└── rules/                          # 用户自定义规则
    └── custom.yaml
```

### 3.2 数据模型

#### config.json
```json
{
  "llm": {
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-xxx",
    "model": "gpt-4o"
  },
  "server": {
    "port": 8080,
    "host": "127.0.0.1"
  },
  "license": {
    "key": "xxx-xxx-xxx",
    "activated_at": "2024-01-01T00:00:00Z",
    "valid_until": "2025-01-01T00:00:00Z"
  },
  "rule_sources": [
    "https://raw.githubusercontent.com/xxx/rules/main/gfw.yaml"
  ]
}
```

#### subscriptions.json
```json
{
  "subscriptions": [
    {
      "id": "uuid-1",
      "name": "机场A",
      "url": "https://xxx.com/sub",
      "enabled": true,
      "last_updated": "2024-01-01T00:00:00Z",
      "node_count": 50
    }
  ]
}
```

#### 版本快照文件命名
```
v_{timestamp}_{description}.yaml
```
- timestamp: Unix 时间戳
- description: 简短描述（AI 生成或手动编辑）

---

## 4. AI 集成设计

### 4.1 Prompt 设计

**System Prompt:**
```
你是一个 Clash 配置专家。用户会描述他们对代理规则的需求，你需要生成 JSON Patch 格式的配置修改。

可用的 proxy-groups:
{动态注入当前配置的 proxy-groups 列表}

可用的 proxies:
{动态注入解析出的节点列表，按地区分组}

规则格式参考:
- DOMAIN-SUFFIX,google.com,代理组名
- DOMAIN-KEYWORD,google,代理组名
- IP-CIDR,8.8.8.8/32,代理组名
- GEOIP,US,代理组名

输出格式要求:
{
  "description": "简短描述这次修改做了什么",
  "operations": [
    {"op": "add", "path": "/rules/0", "value": "DOMAIN-SUFFIX,google.com,Taiwan"},
    {"op": "replace", "path": "/proxy-groups/2/proxies/0", "value": "台湾节点1"}
  ]
}
```

**User Prompt 示例:**
```
让 Google 服务走台湾节点
```

### 4.2 AI 输出处理流程

```
用户输入 -> 构建 Prompt -> 调用 LLM API -> 解析 JSON Patch
-> 应用到当前配置 -> 生成 diff 预览 -> 用户确认 -> 保存新版本
```

### 4.3 错误处理
- JSON 解析失败：提示用户重试
- Patch 应用失败：显示具体错误，保留原配置
- API 调用失败：显示错误信息，支持重试

---

## 5. 版本管理设计

### 5.1 版本创建时机
- AI 修改确认后
- 手动编辑保存后
- 导入配置后

### 5.2 版本操作
- **查看**: 显示该版本的完整内容
- **对比**: 与当前版本或任意版本做 diff
- **回退**: 将该版本设为当前配置
- **删除**: 删除该版本快照

### 5.3 Diff 算法
使用 YAML 感知的 diff 算法：
- 结构化对比（不是纯文本 diff）
- 高亮添加的规则（绿色）
- 高亮删除的规则（红色）
- 高亮修改的值（黄色）

---

## 6. 规则库设计

### 6.1 规则源
使用 [Loyalsoldier/clash-rules](https://github.com/Loyalsoldier/clash-rules) 作为主要规则源。

**默认订阅的规则集：**
| 规则集 | 用途 | URL |
|--------|------|-----|
| `reject` | 广告域名 | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/reject.txt` |
| `proxy` | 代理域名 | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/proxy.txt` |
| `direct` | 直连域名 | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/direct.txt` |
| `gfw` | GFW 封锁 | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/gfw.txt` |
| `greatfire` | GreatFire | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/greatfire.txt` |
| `tld-not-cn` | 非中国顶级域 | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/tld-not-cn.txt` |
| `telegramcidr` | Telegram IP | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/telegramcidr.txt` |
| `cncidr` | 中国 IP | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/cncidr.txt` |
| `lancidr` | 局域网 IP | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/lancidr.txt` |
| `applications` | 需代理的程序 | `https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/applications.txt` |

### 6.2 规则使用方式
在生成的 Clash 配置中使用 `rule-providers` 引用：
```yaml
rule-providers:
  reject:
    type: http
    behavior: domain
    url: "https://cdn.jsdelivr.net/gh/Loyalsoldier/clash-rules@release/reject.txt"
    path: ./ruleset/reject.yaml
    interval: 86400

rules:
  - RULE-SET,reject,REJECT
  - RULE-SET,proxy,Proxy
  - RULE-SET,direct,DIRECT
  - GEOIP,CN,DIRECT
  - MATCH,Proxy
```

### 6.3 规则更新策略
- 应用启动时检查更新（每24小时一次）
- 手动刷新按钮
- 缓存到 `~/.hangar/cache/rules/`

### 6.4 规则合并优先级
1. AI 生成的规则（最高，插入到 rules 顶部）
2. rule-providers 引用的规则集
3. 默认兜底规则

---

## 7. 国际化 (i18n) 设计

### 7.1 支持语言
- 简体中文 (zh-CN) - 默认
- English (en)

### 7.2 实现方案
使用 `react-i18next` 库：

```tsx
// src/i18n/index.ts
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

i18n.use(initReactI18next).init({
  resources: {
    'zh-CN': { translation: require('./locales/zh-CN.json') },
    'en': { translation: require('./locales/en.json') }
  },
  lng: 'zh-CN',
  fallbackLng: 'en'
});
```

### 7.3 翻译文件结构
```
src/i18n/
├── index.ts
└── locales/
    ├── zh-CN.json
    └── en.json
```

### 7.4 翻译键命名规范
```json
{
  "tabs": {
    "main": "主页",
    "subscriptions": "机场管理",
    "settings": "设置"
  },
  "main": {
    "ai_placeholder": "描述你的需求，例如：让 Google 走台湾节点",
    "generate": "生成",
    "server_start": "启动服务器",
    "server_stop": "停止服务器"
  }
}
```

---

## 8. 技术实现

### 8.1 前端
- **框架**: React 19 + TypeScript
- **UI 组件**: 自定义或 shadcn/ui
- **状态管理**: React hooks (useReducer/useState)
- **Diff 显示**: react-diff-viewer 或自定义

### 8.2 后端 (Tauri/Rust)
- **HTTP 服务器**: Axum（保持现有）
- **YAML 处理**: serde_yaml
- **JSON Patch**: json-patch crate
- **HTTP 客户端**: reqwest（LLM API 调用）
- **机器码生成**: machine-uid crate

### 8.3 新增 Tauri Commands
```rust
// AI 相关
#[tauri::command]
fn generate_config_patch(prompt: String) -> Result<PatchResult, Error>

#[tauri::command]
fn apply_patch(patch: JsonPatch) -> Result<(), Error>

// 版本管理
#[tauri::command]
fn list_versions() -> Result<Vec<Version>, Error>

#[tauri::command]
fn get_version(id: String) -> Result<String, Error>

#[tauri::command]
fn rollback_to_version(id: String) -> Result<(), Error>

#[tauri::command]
fn diff_versions(v1: String, v2: String) -> Result<Diff, Error>

// License
#[tauri::command]
fn activate_license(key: String) -> Result<LicenseInfo, Error>

#[tauri::command]
fn check_license() -> Result<LicenseStatus, Error>

// 规则
#[tauri::command]
fn refresh_rules() -> Result<(), Error>

#[tauri::command]
fn list_rule_sources() -> Result<Vec<RuleSource>, Error>

// 编辑器
#[tauri::command]
fn open_in_editor() -> Result<(), Error>
```

---

## 9. 开发阶段规划

### Phase 1: 基础重构
- [ ] 重命名项目为 Hangar
- [ ] 重构数据目录结构
- [ ] 实现新的 Tab 布局
- [ ] 迁移现有机场管理功能

### Phase 2: 版本管理
- [ ] 实现版本快照存储
- [ ] 实现版本列表 UI
- [ ] 实现 diff 显示
- [ ] 实现版本回退

### Phase 3: AI 集成
- [ ] LLM 配置 UI
- [ ] Prompt 设计与调优
- [ ] JSON Patch 生成与应用
- [ ] AI 修改预览与确认流程

### Phase 4: 规则库
- [ ] 打包内置规则
- [ ] 在线规则订阅管理
- [ ] 规则合并逻辑

### Phase 5: License 系统
- [ ] License 验证服务器（单独项目）
- [ ] 客户端验证逻辑
- [ ] 功能限制实现

### Phase 6: 打磨与发布
- [ ] UI/UX 优化
- [ ] 错误处理完善
- [ ] 文档撰写
- [ ] 构建与分发

---

## 10. 开放问题

1. **产品名确认**: Hangar 
2. **License 服务器**:  后续接入
3. **定价策略**: 一次性买断价格Pro功能
4. **多语言**: 支持中英文
5. **自动更新**: 暂时不用支持

---

*文档版本: 1.0*
*最后更新: 2026-02-01*
