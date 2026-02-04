# Hangar CLI Redesign

## 1. Current Implementation
Based on `src-tauri/src/main.rs`, the current CLI structure is:

关于 basic.yml 的说明：
 这个文件 应该放在 src-tauri/resources/目录下
 basic.yml 是没有proxy字段的，

### `sub` (Subscription Management)
Manage Clash subscription links.
- `add <url> [--name <name>]`
  - Adds a new subscription. 
  Update: add命令除了保存到配置之外需要自动下载配置到 .hangar/cache/proxies/目录下; 需要使用 User-Agent: clash-verge/v2.4.5 下载 才能保证下载的是yml,或者文件File: application/octet-stream但是内容是yml

- `list`
  - Lists all saved subscriptions with ID, Name, Node count, and Enabled status.
- `remove <id>`
  - Removes a subscription by ID or index.
- `enable <id>`
  - Enables a subscription. The `<id>` parameter can be:
    1. Subscription name (recommended, easiest to remember)
    2. Full UUID
    3. Numeric index (from `list` command)
  - Enabled subscriptions will be included in merge operations and auto-updates.
- `disable <id>`
  - Disables a subscription. The `<id>` parameter can be:
    1. Subscription name (recommended, easiest to remember)
    2. Full UUID
    3. Numeric index (from `list` command)
  - Disabled subscriptions will be skipped in merge operations and auto-updates.
- `fetch <id>`
  - Fetches/updates nodes for a specific subscription ID.
  - 更新nodes之后需要自动保存到 .hangar/cache/proxies/目录下
  fix : 需要使用 User-Agent: clash-verge/v2.4.5 下载 才能保证下载的是yml,或者文件File: application/octet-stream但是内容是yml
  修改 fetch命令为 merge , 使用已经下载好的yml文件进行合并，合并basic.yml和下载的yml文件
- Merges all subscriptions with the basic config and generates `current.yaml`.
merge到逻辑主要是生成以国家命名的group，然后再把所有的groups添加到 groups.yml中配置的所有groups下
basic.yml中配置的rules中的target group都是以 groups.yml中配置的groups为基础
- **Note**: This is the "build" or "generate" step.


Update: 添加一个 editor命令，用默认编辑器打开 current.yaml 允许手动编辑
editor basic 命令用默认编辑期打开 basic.yaml 手动修改后，需要重新运行 merge 命令

### `serve`
Start the configuration server.
- `--port <port>` (default: 8080)
- `--host <host>` (default: 127.0.0.1)
- `--interval <seconds>` (default: 0, disabled)
  - When set to a value > 0, enables auto-refresh of all subscriptions and merges new current.yaml
  - Server automatically watches current.yaml for changes and reloads the configuration dynamically
  - Any manual edits or automated updates to current.yaml will be detected and reloaded in real-time
- `--daemon` / `-d` (default: false)
  - Run server in daemon mode (background process)
  - Logs are written to `~/.hangar/server.log`
  - PID is saved to `~/.hangar/server.pid`
  - Use `hangar stop`, `hangar status`, etc., to manage the daemon

Example usage:
```bash
# Start server in foreground
hangar serve --port 8080

# Start server in background (daemon mode)
hangar serve --daemon --port 8080 --interval 300

### `stop`, `restart`, `status`, `logs`
Manage the background server.
- `stop`: Gracefully stops the daemonized server.
- `restart`: Restarts the daemonized server (using default settings).
- `status`: Checks if the server is running.
- `logs [-f]`: Shows server logs (with optional real-time follow).

Example usage:
```bash
# Start server in background (daemon mode)
hangar serve --daemon --port 8080 --interval 300
# or use the 'start' alias
hangar start --daemon --port 8080 --interval 300

# Management commands
hangar status   # Check status
hangar logs -f  # Follow logs
hangar restart  # Restart daemon
hangar stop     # Stop daemon
```

### `ai`
AI-powered configuration modification.
- `<prompt>`
  - Sends a natural language prompt to the AI to modify the config.
  - Applies changes and creates a backup.
AI命令对current.yaml进行修改， 然后生成新的current.yaml 
也有可能修改新的basic.yml,这样方式subscription更新或者新的 AI命令执行后有些规则仍然在basic.yml中存在。



### `history`
Version history management for configurations.
- `list`
  - Lists all configuration snapshots.
- `rollback <id>`
  - Reverts `current.yaml` to a specific version.
- `diff <v1> [v2]`
  - Shows the diff between version `v1` and `v2` (or current).

### `config`
Application configuration (LLM settings).
- `--api-key <key>`
- `--base-url <url>`
- `--model <model>`

## 2. Redesign Goals
- [ ] Review command hierarchy
- [ ] Improve consistency (e.g., `config` flags vs subcommands)
- [ ] Clarify command names (e.g., `update` vs `fetch`)
