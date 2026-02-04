# Hangar Landing Page 设计规格文档

## 项目概述

**项目名称：** Hangar  
**定位：** 智能代理订阅管理工具  
**目标用户：** 高级技术用户、开发者、多机场订阅用户

---

## 设计系统

### 颜色方案（严格避免蓝色/紫色）

**主色调：**
- 深色背景：`#0A0A0A`（主背景）、`#121212`（卡片背景）
- 强调色：`#FF6B35`（橙红色 - 主要 CTA）
- 次要强调：`#FFA500`（金橙色 - 悬停状态）
- 科技绿：`#00FF88`（状态指示、图标点缀）
- 中性色：`#E0E0E0`（主文本）、`#888888`（次要文本）

**渐变组合：**
- Hero 渐变：`linear-gradient(135deg, #0A0A0A 0%, #1A0F0F 50%, #0A0A0A 100%)`
- 卡片光晕：`radial-gradient(circle at center, rgba(255,107,53,0.1) 0%, transparent 70%)`

### 字体系统

**中文字体栈：**
```css
font-family: 'PingFang SC', 'Microsoft YaHei', 'Helvetica Neue', sans-serif;
```

**字体比例：**
- Hero 标题：72px / 600 (bold)
- H2 标题：48px / 600
- H3 标题：32px / 600
- 正文：18px / 400
- 小字：14px / 400

### 间距系统

**8px 基准网格：**
- 容器最大宽度：1280px
- Section 间距：120px (vertical)
- 卡片间距：32px
- 内边距：24px (cards), 40px (sections)

---

## Hero Section（视频驱动）

### 视频规格
- **分辨率：** 1920x1080 (1080p)
- **时长：** 6-10秒循环
- **风格：** 高科技、数据流、网络连接可视化
- **提示词增强：** "Abstract network nodes connecting in 3D space, cinematic tech visualization, orange and green light trails, data packets flowing through digital tunnels, shot on Arri Alexa, 35mm lens f/1.8, slow motion 120fps, 8k quality, dark ambient background with warm accent lighting, hyper-realistic render"

### Hero 布局
- 100vh 全屏视频背景（自动播放、静音、循环）
- 居中大标题：**"Hangar"** + 副标题
- 描述文本：核心价值主张
- CTA 按钮：GitHub 链接 + 下载按钮（占位）
- 无静态图片占位符（纯视频 + 渐变遮罩）

---

## 功能特性区（Features）

### 6个核心功能卡片

每个卡片包含：
- SVG 图标（Lucide Icons）
- 功能标题
- 简短描述（1-2句）
- 悬停效果：轻微上浮 + 边框光晕

**卡片列表：**
1. 多订阅合并 🔗
2. 智能区域识别 🌏
3. 智能分组 🎯
4. 内置 HTTP 服务器 🚀
5. 版本管理 📝
6. AI 驱动配置 🤖（即将推出标签）

**布局：** 3列网格（桌面）→ 1列（移动端）

---

## 技术栈区

### 技术标签云
- React 19
- TypeScript
- Rust
- Tauri 2.0
- Vite
- Axum
- Tailwind CSS

**视觉风格：**
- 标签：深色背景 + 橙色边框
- 悬停：边框发光效果
- 布局：Flex wrap 居中排列

---

## 使用流程区（How it Works）

### 4步流程
1. 添加订阅源
2. 自动合并节点
3. 智能分组配置
4. 一键订阅使用

**视觉设计：**
- 时间轴式布局（垂直/水平响应）
- 步骤编号圆圈：橙色边框 + 绿色连接线
- 每步配图标

---

## Footer

**内容：**
- GitHub 链接
- MIT License 声明
- Made with ❤️ by Matrix Agent
- Copyright 2026

---

## 技术实现策略

### 前端栈
- **构建工具：** Vite 6.x
- **框架：** 纯 HTML + CSS（或轻量 JS）
- **样式：** Tailwind CSS 或内联 CSS
- **图标：** Lucide SVG Icons

### 视觉资产
1. **Hero Video：** `hero_network.mp4` (1080p, 循环)
2. **App Icon：** `icon-512.png`, `icon-256.png`, `icon-128.png`, `icon-64.png`

### 响应式断点
- Desktop: 1280px+
- Tablet: 768px - 1279px
- Mobile: < 768px

---

## 动画与交互

### 关键动画
- **Hero 入场：** 标题渐入 + 上浮（1.2s ease-out）
- **功能卡片：** 滚动触发渐入（stagger 效果）
- **按钮悬停：** 缩放 1.05 + 光晕扩散
- **视频背景：** 轻微 Ken Burns 缩放（1.0 → 1.05 over 20s）

### 性能优化
- 视频懒加载（intersection observer）
- 图片 WebP 格式 + 降级
- 关键 CSS 内联
- 最小化外部依赖

---

## 部署清单

✅ 资产生成（Video + Icons）  
✅ HTML/CSS/JS 代码生成  
✅ Vite 项目初始化  
✅ Playwright 路径验证  
✅ 生产构建（npm run build）  
✅ 部署（deploy 工具）  
✅ 提供线上 URL

---

## 品质标准

- ✅ 零蓝色/紫色元素
- ✅ 1080p Hero Video
- ✅ 无 404 媒体错误
- ✅ 100% 中文内容
- ✅ 移动端完美适配
- ✅ 加载时间 < 3s (3G)
