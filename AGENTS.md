# Agent Guidelines for proxy-sub-manager

This document provides coding agents with essential information about this project's structure, conventions, and workflows.

## Project Overview

**Type:** Desktop application (Tauri 2.0)  
**Frontend:** React 19 + TypeScript 5.8 + Vite 7  
**Backend:** Rust (2021 edition) + Tokio + Axum  
**Purpose:** Proxy subscription aggregation and HTTP server management

## Build, Lint, and Test Commands

### Development
```bash
npm run tauri dev          # Run full Tauri app in dev mode (frontend + backend)
npm run dev                # Run Vite dev server only (frontend)
cargo run --manifest-path src-tauri/Cargo.toml  # Run Rust backend only
```

### Build
```bash
npm run build              # Build frontend (TypeScript check + Vite build)
npm run tauri build        # Build production app bundle
cargo build --release --manifest-path src-tauri/Cargo.toml  # Build Rust backend
```

### Linting and Formatting
```bash
tsc --noEmit               # TypeScript type checking
cargo clippy --manifest-path src-tauri/Cargo.toml  # Lint Rust code
cargo fmt --manifest-path src-tauri/Cargo.toml     # Format Rust code
```

### Testing
```bash
cargo test --manifest-path src-tauri/Cargo.toml              # Run all Rust tests
cargo test --manifest-path src-tauri/Cargo.toml test_name    # Run single test
cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture  # Show test output
```

**Note:** No test framework currently configured for TypeScript/React.

## Code Style Guidelines

### TypeScript/React

#### Imports
Order imports in groups with blank lines between:
```typescript
// 1. React and external libraries
import React from "react";
import { useState } from "react";

// 2. Tauri APIs
import { invoke } from "@tauri-apps/api/core";

// 3. Local components
import App from "./App";

// 4. CSS (last)
import "./App.css";
```

#### Naming Conventions
- **Components:** PascalCase (`App`, `ProxyManager`)
- **Functions:** camelCase (`startServer`, `fetchProxies`)
- **Variables:** camelCase (`serverStatus`, `proxyList`)
- **State setters:** `set` prefix + PascalCase (`setServerStatus`)
- **Files:** `.tsx` for components, `.ts` for utilities

#### Types
- Use explicit type annotations on Tauri `invoke` calls:
  ```typescript
  const result = await invoke<string>("start_proxy_server");
  ```
- Enable strict mode (already configured in `tsconfig.json`)
- Prefer type inference where obvious

#### Components
- Use functional components with hooks
- No React.FC type annotation needed
- Wrap root in `React.StrictMode`
- Bilingual support: UI text in Chinese with English fallbacks

### Rust

#### Imports
Order imports in three groups:
```rust
// 1. Crate-internal modules
use crate::types::{ClashConfig, Subscription};
use crate::config::load_app_config;

// 2. External crates (alphabetical)
use anyhow::Result;
use axum::{Router, routing::get};
use serde::{Deserialize, Serialize};

// 3. Standard library
use std::collections::HashMap;
use std::sync::Arc;
```

#### Naming Conventions
- **Structs/Enums:** PascalCase (`ProxyNode`, `ClashConfig`)
- **Functions:** snake_case (`load_app_config`, `merge_configs`)
- **Variables:** snake_case (`all_proxies`, `region_map`)
- **Constants:** SCREAMING_SNAKE_CASE (`SERVER_RUNNING`, `DEFAULT_PORT`)
- **Files:** snake_case (`proxy.rs`, `subscription.rs`)

#### Types
- Return `Result<T>` or `anyhow::Result<T>` for fallible operations
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data types
- Serde attributes for field customization:
  ```rust
  #[serde(rename = "type")]
  proxy_type: String,
  ```
- Prefer owned types over lifetimes for simplicity
- Mark public APIs with `pub`

#### Error Handling
- **Always** return `Result` types for fallible operations
- Use `?` operator for error propagation
- Convert errors with context:
  ```rust
  .map_err(|e| anyhow!("加载配置失败: {}", e))?
  ```
- User-facing error messages in Chinese
- Use `.expect()` only for initialization errors that should panic

#### Async Patterns
- Use `async fn` for I/O operations
- `tokio::spawn` for background tasks
- `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for shared state
- Always `.await` async operations

#### Documentation
- Use `///` for public API documentation
- Document complex logic with inline comments
- Example:
  ```rust
  /// Fetches and merges all proxy subscriptions
  /// Returns ClashConfig with merged proxy nodes
  pub async fn merge_configs() -> Result<ClashConfig> { ... }
  ```

## Project Structure

```
proxy-sub-manager/
├── src/                    # Frontend React/TypeScript
│   ├── App.tsx            # Main React component
│   ├── main.tsx           # Entry point
│   └── App.css            # Styles
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Tauri commands & exports
│   │   ├── main.rs        # Application entry
│   │   ├── types.rs       # Type definitions
│   │   ├── config.rs      # Config loading/saving
│   │   ├── proxy.rs       # Proxy logic
│   │   ├── subscription.rs # Subscription fetching
│   │   └── server.rs      # HTTP server (Axum)
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── _docs/                 # Documentation
└── public/                # Static assets
```

## Key Configuration Files

- **tsconfig.json:** Strict TypeScript with ES2020 target
- **vite.config.ts:** Dev server on port 1420, HMR for Tauri
- **Cargo.toml:** Rust edition 2021, full Tokio features
- **tauri.conf.json:** 800x600 window, localhost:1420 dev URL

## Common Patterns

### Tauri Commands (Rust)
```rust
#[tauri::command]
async fn my_command(param: String) -> Result<String, String> {
    do_something(param)
        .await
        .map_err(|e| e.to_string())
}
```

### Invoking Tauri Commands (TypeScript)
```typescript
const result = await invoke<string>("my_command", { param: "value" });
```

### Shared State (Rust)
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    static ref STATE: Arc<RwLock<MyState>> = Arc::new(RwLock::new(MyState::default()));
}
```

## Important Notes

- **No ESLint/Prettier:** Manual code formatting for TypeScript
- **No Tests Yet:** Write tests when adding critical features
- **Bilingual UI:** Use Chinese for user-facing text
- **Sensitive Data:** Never commit `subs.txt` (in `.gitignore`)
- **Port 1420:** Hardcoded for dev server (Vite + Tauri)

## When Making Changes

1. **Frontend changes:** Run `npm run dev` for hot reload
2. **Backend changes:** Run `npm run tauri dev` to test integration
3. **Type checking:** Run `tsc --noEmit` before committing
4. **Rust linting:** Run `cargo clippy` and fix warnings
5. **Format Rust:** Run `cargo fmt` before committing
6. **Test builds:** Run `npm run build` to ensure production readiness

## Dependencies

Keep dependencies up to date:
- Check frontend: `npm outdated`
- Check backend: `cargo outdated` (requires cargo-outdated)
- Update frontend: `npm update`
- Update backend: Edit `Cargo.toml` and run `cargo update`
