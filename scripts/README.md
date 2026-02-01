# 打包脚本说明

本目录包含用于构建和打包应用的脚本。

## 脚本列表

### 1. `simple-dmg.sh` - 简化 DMG 打包（推荐）

创建一个简单的 DMG 安装镜像。

**使用方法:**
```bash
npm run build:dmg
# 或
./scripts/simple-dmg.sh
```

**特点:**
- ✅ 快速可靠
- ✅ 包含 Applications 快捷方式
- ✅ 包含安装说明文件
- ✅ 自动压缩优化

**产物:**
- `src-tauri/target/release/bundle/dmg/hangar_0.1.0_custom_arm64.dmg`

---

### 2. `build-dmg.sh` - 高级 DMG 打包

创建带有自定义窗口布局的高级 DMG。

**使用方法:**
```bash
npm run build:dmg:fancy
# 或
./scripts/build-dmg.sh
```

**特点:**
- ✅ 自定义窗口大小和图标位置
- ✅ 美化的窗口布局
- ✅ 包含 README 文件
- ⚠️  需要更多时间

**产物:**
- `src-tauri/target/release/bundle/dmg/hangar_0.1.0_arm64.dmg`

---

### 3. `quick-build.sh` - 一键完整构建

运行完整的构建流程：测试 → 构建 → 打包。

**使用方法:**
```bash
npm run build:quick
# 或
./scripts/quick-build.sh
```

**流程:**
1. TypeScript 类型检查
2. Rust 单元测试
3. Clippy 代码检查
4. 构建 Tauri 应用
5. 创建 DMG 安装包

---

### 4. `create-dmg-background.sh` - 背景图生成

生成 DMG 自定义背景图（可选）。

**使用方法:**
```bash
./scripts/create-dmg-background.sh
```

**要求:**
- 需要安装 ImageMagick: `brew install imagemagick`

---

## 完整构建流程

### 开发测试
```bash
# 1. 开发模式运行
npm run tauri dev

# 2. TypeScript 检查
npx tsc --noEmit

# 3. Rust 测试
cargo test --manifest-path src-tauri/Cargo.toml
```

### 生产构建
```bash
# 方法 1: 分步构建（推荐）
npm run tauri build    # 构建应用
npm run build:dmg      # 创建 DMG

# 方法 2: 一键构建
npm run build:quick    # 测试 + 构建 + 打包

# 方法 3: 构建所有
npm run build:all      # 构建 + DMG
```

---

## 构建产物

构建完成后，产物位于：

```
src-tauri/target/release/bundle/
├── macos/
│   └── hangar.app          # macOS 应用包
└── dmg/
    ├── hangar_0.1.0_aarch64.dmg       # Tauri 自动生成
    └── hangar_0.1.0_custom_arm64.dmg  # 自定义脚本生成
```

---

## 发布检查清单

在发布新版本前，请确保：

- [ ] 更新 `package.json` 中的版本号
- [ ] 更新 `src-tauri/tauri.conf.json` 中的版本号
- [ ] 运行所有测试: `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] TypeScript 检查: `npx tsc --noEmit`
- [ ] Clippy 检查: `cargo clippy --manifest-path src-tauri/Cargo.toml`
- [ ] 构建应用: `npm run tauri build`
- [ ] 创建 DMG: `npm run build:dmg`
- [ ] 测试 DMG 安装
- [ ] 测试应用功能
- [ ] 更新 CHANGELOG.md

---

## 常见问题

### Q: DMG 创建失败提示 "Resource temporarily unavailable"
**A:** 运行以下命令清理挂载的镜像：
```bash
hdiutil detach /Volumes/代理订阅管理器 -force
rm -f /tmp/hangar_tmp.dmg
```

### Q: 应用无法打开，提示"来自身份不明的开发者"
**A:** 在终端运行：
```bash
xattr -cr /Applications/hangar.app
```

### Q: 如何减小 DMG 文件大小？
**A:** 已经使用 `zlib-level=9` 最高压缩级别。如需进一步优化，考虑：
- 移除不必要的资源文件
- 优化图片和图标
- 使用 `strip` 命令移除调试符号

### Q: 如何自定义 DMG 图标？
**A:** 编辑 `src-tauri/icons/icon.icns` 文件，然后重新构建。

---

## 脚本维护

如需修改打包脚本：

1. **测试前备份**: 复制脚本文件备份
2. **逐步测试**: 先在临时目录测试
3. **版本控制**: 提交前确保脚本可执行
4. **文档更新**: 修改后更新本文档

---

## License

MIT
