# Typstify 依赖升级和 UI 框架迁移报告

## 完成日期
2024-11-13

## 升级概述

### 1. 依赖更新（全部升级到最新版本）

#### Rust 依赖更新
- **Leptos**: 0.8.6 → 0.8.12
- **Regex**: 1.11.1 → 1.12.2
- **Serde**: 1.0.219 → 1.0.228
- **Thiserror**: 1.0.69 → 2.0.17
- **Chrono**: 0.4.38 → 0.4.42
- **Tantivy**: 0.24 → 0.25
- **Pulldown-cmark**: 0.12 → 0.13
- **Clap**: 4.5.23 → 4.5.51
- **Typst-syntax**: 0.13.1（保持最新）
- 以及 145+ 其他依赖包自动更新

#### 新增依赖
- **Singlestage**: v0.3.9（新 UI 框架）
  - 启用功能：accordion, alert, avatar, badge, breadcrumb, button, card, checkbox, csr, dialog, dropdown, input, label, link, pagination, radio, select, separator, sidebar, skeleton, slider, switch, table, tabs, textarea, theme_provider, tooltip

#### JavaScript 依赖清理
- **移除**: DaisyUI 5.0.50
- **移除**: @tailwindcss/typography
- **保留**: @tailwindcss/cli 4.1.17, Tailwind CSS 4.1.17
- **构建工具**: Bun 1.3.2

### 2. UI 框架迁移：DaisyUI → Singlestage

#### 为什么选择 Singlestage？
- 现代化的 Rust/WASM 组件库
- 与 Leptos 完美集成
- 基于 Tailwind CSS 变量系统
- 支持亮色/暗色主题
- 更轻量，无 jQuery 依赖
- 参考：https://singlestage.doordesk.net/

#### CSS 架构重构
**旧架构（DaisyUI）**:
- 708 行 CSS，大量 DaisyUI 特定类
- 依赖 `bg-base-200`, `text-base-content` 等
- 主题系统绑定到 DaisyUI

**新架构（Singlestage）**:
- 简化为 400+ 行语义化 CSS
- HSL 变量系统：`--background`, `--foreground`, `--primary` 等
- 标准 Tailwind 实用类
- 完整的亮色/暗色主题支持

#### 迁移的 CSS 类映射
```
DaisyUI                 → Singlestage/Tailwind
-------------------------------------------------
bg-base-100            → bg-background
bg-base-200            → bg-muted
bg-base-300            → bg-muted / bg-card
text-base-content      → text-foreground
border-base-300        → border
primary/[opacity]      → primary with opacity-[value]
```

### 3. 组件样式更新

#### 保留的自定义组件
所有 Typst 特定样式保持不变：
- `.typst-table` - Typst 表格样式
- `.typst-table-header` - 表格标题
- `.typst-table-cell` - 表格单元格
- `.typst-line` - Typst 分隔线
- `.typst-link` - Typst 链接

#### 更新的布局组件
- **Sidebar**: 使用 `hsl(var(--card))` 背景
- **Navigation**: 使用 `hsl(var(--primary))` 高亮
- **Content area**: 使用 `hsl(var(--background))` 背景
- **Code blocks**: 使用 `hsl(var(--muted))` 背景

### 4. 构建验证

✅ **CSS 编译**: 成功（44ms）
```bash
$ bun run build
✓ Tailwind CSS v4.1.17 - Done in 44ms
```

✅ **Rust 编译**: 成功（2m 05s）
```bash
$ cargo build --release
Finished `release` profile [optimized] target(s) in 2m 05s
```

✅ **站点生成**: 成功
```bash
$ ./target/release/typstify-ssg
Loaded: contents/rust-guide.md
Loaded: contents/getting-started/quick-start.typ
Loaded: contents/getting-started/installation.typ
Loaded: contents/javascript-modern.md
Loaded: contents/getting-started.typ
```

✅ **生成的文件**:
- getting-started.html (11KB)
- index.html (8.0KB)
- installation.html (15KB)
- javascript-modern.html (13KB)
- quick-start.html (15KB)
- rust-guide.html (9.8KB)
- 以及其他页面

### 5. 文件变更

#### 修改的文件
1. `/Cargo.toml` - 更新 workspace 依赖，添加 singlestage
2. `/typstify-ssg/Cargo.toml` - 更新项目依赖
3. `/package.json` - 移除 DaisyUI 和 typography 插件
4. `/tailwind.config.js` - 配置 singlestage 颜色系统
5. `/style/input.css` - 完全重写为 singlestage 兼容版本

#### 备份文件
- `/style/input.css.old` - 原 DaisyUI 版本（已备份）

### 6. 功能验证

✅ **Typst 元素渲染**（之前修复的功能保持正常）:
- `#line(length: 100%)` → `<hr class="typst-line">`
- `#table()` 语法 → HTML 表格（支持 2-3 列）
- `#link()` 语法 → `<a>` 标签
- 中文字符支持（UTF-8 安全）

✅ **响应式设计**:
- 桌面端：固定侧边栏（18rem 宽）
- 移动端：可折叠侧边栏

✅ **主题支持**:
- 亮色模式：完整样式
- 暗色模式：完整样式
- CSS 变量平滑过渡

### 7. 性能提升

- **CSS 文件大小**: 708 行 → ~400 行（减少 43%）
- **编译时间**: CSS 编译 <50ms
- **依赖清理**: 移除 2 个未使用的 npm 包
- **Rust 编译**: 保持稳定（~2 分钟）

### 8. 下一步建议

#### 可选优化
1. 考虑使用 Singlestage 组件替换现有 HTML 元素
   - Button, Card, Alert 等可以用 Singlestage 组件
   - 更好的可访问性和一致性

2. 添加更多主题变量
   - 可以扩展颜色方案
   - 支持更多主题变体

3. 集成搜索功能
   - 使用 Tantivy 0.25 的新功能
   - 优化搜索索引

#### 维护建议
- 定期运行 `cargo update` 保持依赖最新
- 监控 Singlestage 更新（当前 0.3.9）
- 关注 Leptos 0.8.x 更新

### 9. 回滚方案

如需回滚到 DaisyUI 版本：
```bash
# 恢复 CSS
cp style/input.css.old style/input.css

# 恢复 package.json
bun add daisyui@5.0.50 @tailwindcss/typography -d

# 恢复 tailwind.config.js
# 手动添加回 daisyui plugin 和主题配置

# 从 Cargo.toml 移除 singlestage
# 运行 cargo update
```

### 10. 总结

✅ **完成的任务**:
- 所有 Cargo 依赖更新到最新版本
- 成功迁移到 Singlestage UI 框架
- CSS 完全重写为语义化、可维护版本
- 构建系统验证通过
- 所有现有功能保持正常

✅ **收益**:
- 更现代的技术栈
- 更好的 Rust/WASM 集成
- 更清晰的 CSS 架构
- 更小的依赖树
- 更好的主题支持

✅ **兼容性**:
- 所有 Typst 渲染功能保持不变
- HTML 结构保持兼容
- 搜索功能正常
- RSS feed 生成正常

---

**升级完成时间**: 2024-11-13 18:08 (UTC+8)
**测试状态**: 全部通过 ✅
