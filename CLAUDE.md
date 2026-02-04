# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hangar is a Tauri-based desktop application for managing proxy subscriptions (机场). It merges multiple Clash subscription sources and provides an AI-powered interface to modify Clash configurations using natural language.

**Tech Stack:**
- Frontend: React 19 + TypeScript + Vite
- Backend: Rust + Tauri 2.0
- HTTP Server: Axum
- Build Tool: Bun (package manager)

## Development Commands

### Running the Application

```bash
# Development mode (hot reload)
bun run dev
# or
bun run tauri:dev

# Frontend only (for UI development)
bun run dev:frontend
```

### Building

```bash
# Full production build
bun run tauri:build

# Quick build with tests and DMG packaging
bun run build:quick

# Build everything (app + DMG)
bun run build:all

# Create DMG installer only (simple version, recommended)
bun run build:dmg

# Create DMG with fancy layout
bun run build:dmg:fancy
```

### Testing

```bash
# TypeScript type checking
npx tsc --noEmit

# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Rust linting
cargo clippy --manifest-path src-tauri/Cargo.toml
```

### CLI Tool

The `hangar` binary itself serves as a full-featured CLI tool for managing subscriptions, configurations, and the server without the UI.

```bash
# Run CLI commands via Cargo
cargo run --manifest-path src-tauri/Cargo.toml -- sub list
cargo run --manifest-path src-tauri/Cargo.toml -- update

# Build standalone CLI binary (same as app binary)
cargo build --manifest-path src-tauri/Cargo.toml --release
# Binary location: src-tauri/target/release/hangar
```

Available command groups: `sub` (manage subscriptions), `update` (merge config), `serve` (start server), `ai` (LLM modification), `history` (versioning), and `config` (settings).

## Architecture

### Frontend Structure

- **App.tsx**: Main application component with tab-based UI
  - Tab 1 (Main): AI config generation, version history, server control
  - Tab 2 (Subscriptions): Manage proxy subscriptions
  - Tab 3 (Settings): LLM config, server settings, rule sources
- **main.tsx**: Application entry point
- Single-page application using React hooks for state management

### Backend Structure (src-tauri/src/)

- **main.rs**: Tauri application entry point
- **lib.rs**: Core library with Tauri commands and business logic
- **server.rs**: Axum HTTP server for serving Clash configurations
- **subscription.rs**: Subscription fetching and parsing logic
- **proxy.rs**: Proxy node parsing and region detection
- **config.rs**: Clash configuration generation and merging
- **types.rs**: Shared type definitions
- **bin/cli.rs**: Standalone CLI tool for testing

### Data Architecture

The application uses `~/.hangar/` as the data directory:

```
~/.hangar/
├── config.json              # App settings (LLM, server, license)
├── subscriptions.json       # Subscription list
├── current.yaml            # Active Clash config
├── cache/
│   ├── proxies/           # Cached proxy nodes by subscription
│   └── rules/             # Cached rule sets (builtin + remote)
├── versions/              # Configuration version snapshots
│   └── v_{timestamp}_{description}.yaml
└── rules/
    └── custom.yaml        # User custom rules
```

### Key Features

1. **Subscription Merging**: Fetches multiple Clash subscriptions and merges proxies
2. **Auto Region Detection**: Automatically detects node regions from names (HK, US, JP, TW, SG, etc.)
3. **Smart Grouping**: Creates regional proxy groups and service-specific groups (ChatGPT, Gemini)
4. **HTTP Server**: Serves merged config at `http://127.0.0.1:PORT/config` for Clash clients
5. **Version Management**: Snapshots configurations with diff and rollback capabilities
6. **AI Integration** (planned): Natural language config modifications using LLM

### Tauri Commands

The frontend communicates with Rust backend via these commands:
- `add_subscription(name, url)`: Add new subscription
- `remove_subscription(id)`: Remove subscription
- `list_subscriptions()`: Get all subscriptions
- `fetch_subscription(id)`: Fetch and parse subscription
- `get_merged_config()`: Get merged Clash config
- `start_server(port)`: Start HTTP config server
- `stop_server()`: Stop HTTP server

### Region Detection

The proxy parser (`proxy.rs`) detects regions from node names using patterns:
- Chinese: 香港, 台湾, 日本, 新加坡, 美国, 英国, etc.
- English: HK, TW, JP, SG, US, UK, KR, DE, CA, etc.
- Aliases: 台 (TW), 狮城 (SG), etc.

Nodes are automatically prefixed with `[机场名]` to identify their source.

### Proxy Groups

Generated configs include:
- **Regional groups**: One per detected region (e.g., "HK 地区", "US 地区")
- **节点选择**: All available proxies
- **ChatGPT**: US, UK, SG, TW nodes
- **Gemini**: US, UK, SG, HK, TW nodes

### Build Scripts (scripts/)

- `simple-dmg.sh`: Creates simple DMG with Applications shortcut (recommended)
- `build-dmg.sh`: Creates DMG with custom window layout
- `quick-build.sh`: Full build pipeline (tests → build → package)
- Output: `src-tauri/target/release/bundle/dmg/*.dmg`

## Planned Features (from SPEC.md)

1. **AI Config Modification**: Use LLM to modify Clash rules via natural language prompts
2. **Version Control**: Full config history with diff visualization and rollback
3. **Rule Library**: Integration with Loyalsoldier/clash-rules for rule-providers
4. **Manual Editing**: Open config in system editor with auto-reload
5. **License System**: Activation and validation (future phase)
6. **i18n**: Bilingual support (zh-CN and English)

## Development Notes

- Use `bun` for package management (not npm/yarn)
- Rust code uses `tokio` for async runtime
- HTTP client: `reqwest` for fetching subscriptions
- YAML parsing: `serde_yaml`
- The CLI tool is useful for rapid iteration without starting the full UI
- Build artifacts are in `src-tauri/target/release/bundle/`

## Version Management

When updating versions:
1. Update `package.json` version
2. Update `src-tauri/tauri.conf.json` version
3. Update `src-tauri/Cargo.toml` version
4. Run full test suite before building
5. Update CHANGELOG.md (when it exists)
