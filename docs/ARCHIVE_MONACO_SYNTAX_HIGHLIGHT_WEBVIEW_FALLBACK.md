# [已归档] Monaco 语法高亮 WebView 降级方案

> **状态**: 已归档。Monaco Editor 已于 2026-05-04 彻底移除，改用 CodeMirror 6（详见 CODEMIRROR6_INTEGRATION.md）。
> .tmp-monaco/ 临时目录已清理。
> 本文档保留作为历史问题排查参考。

## 问题描述

Monaco Editor v0.52 在 MAUI BlazorWebView Android 环境中，C/C++ 语法高亮完全失效，所有代码文本显示为灰色。编辑器其他功能（光标、选择、背景色）正常。

## 环境

- **平台**: .NET 10 MAUI Blazor Hybrid Android
- **WebView**: Android System WebView (Chromium-based)
- **协议**: `file://`（本地静态资源）
- **Monaco 版本**: 0.52.2
- **语言**: C/C++ (`cpp`)

## 根本原因分析

Monaco 的语法高亮依赖两条链路：

1. **Tokenizer**: 将源代码拆分为带类型的 token（`keyword`, `string`, `number`, `comment` 等）
2. **Theme CSS 注入**: 将 token 类型映射为具体颜色，通过动态 `<style>` 标签或 `CSSStyleSheet` API 注入到文档

在 WebView `file://` 协议下，以下环节出现问题：

| 环节 | 现象 | 验证结果 |
|------|------|----------|
| AMD Loader 加载 `vs/basic-languages/cpp/cpp` | 外部模块加载静默失败 | 手动 inline Monarch tokenizer 后仍无颜色 |
| Dummy Blob Worker | 阻止了 Monaco 崩溃，但 language worker 不可用 | Worker 不阻塞基础高亮 |
| Theme CSS 变量注入 | `vs-dark` / 自定义 `cide-dark` 的 token 颜色规则未生效 | `editor.background`/`editor.foreground` 工作，但 token 规则不工作 |
| `renderingMode: 0` 强制 DOM 渲染 | 无效果 | Monaco 0.52 可能已移除该选项 |

结论：**Monaco 0.52 的 theme/token CSS 注入机制在 Android WebView `file://` 环境下存在兼容性问题**，无法通过配置修复。

## 已尝试的修复（均未成功）

1. **手动注册 Monarch tokenizer** — inline 完整的 C/C++ tokenizer 规则
2. **自定义主题 `cide-dark`** — 显式定义 `keyword`/`string`/`number`/`comment` 颜色
3. **Dummy Blob Worker** — 避免 `getWorker` 加载外部文件报错
4. **Warm-up tokenization** — 在创建 editor 前预执行 `monaco.editor.tokenize()`
5. **延迟 re-tokenize** — 创建 editor 后 300ms 切换 `plaintext -> cpp` 强制刷新
6. **`renderingMode: 0`** — 尝试强制 DOM 渲染模式

## 最终方案：自动降级到 SimpleEditor

在 `monaco-interop.js` 中增加 tokenizer 健康检查：创建 editor 后 600ms，用 `monaco.editor.tokenize('int main() { return 0; }', 'cpp')` 验证是否产出了非空 token。如果 tokenizer 失效，通过 `dotNetRef.invokeMethodAsync('OnMonacoInitFailed', ...)` 通知 Blazor 层切换为 `SimpleEditor`。

### SimpleEditor 增强点

- **叠加层渲染**: `<textarea>`（透明文字，负责输入和光标）+ `<pre>`（彩色高亮层，pointer-events: none）
- **C 语言语法高亮**（逐词分析，支持）：
  - 关键字（`int`, `for`, `return` 等）→ 蓝色 `#569CD6`
  - 字符串/字符字面量 → 橙色 `#CE9178`
  - 数字（十进制/十六进制/浮点）→ 绿色 `#B5CEA8`
  - 单行注释 `//` → 绿色 `#6A9955`
  - 预处理器指令 `#include` 等 → 紫色 `#C586C0`
- **实时更新**: `@oninput` 触发逐行重新高亮
- **明暗主题**: 通过 `.dark`/`.light` CSS 类切换 token 颜色

## 相关文件修改

| 文件 | 修改内容 |
|------|----------|
| `wwwroot/js/monaco-interop.js` | 添加 warm-up tokenization、延迟 re-tokenize、tokenizer 健康检查（失败时回调 `OnMonacoInitFailed`） |
| `Components/Editor/SimpleEditor.razor` | 重写为叠加层编辑器，添加 C 语言语法高亮逻辑（`HighlightCode` / `HighlightLine` / `HighlightTokens`） |
| `wwwroot/app.css` | 修复 `@media (max-width: 600px)` 中 `.status-bar` 的 `padding-bottom` 被错误覆盖为 `4px` 的 bug（改为 `36px`） |

## 验证结果

- ✅ SimpleEditor 关键字高亮正常（`int`/`for`/`return` 蓝色）
- ✅ 字符串高亮正常（`"%d"` 橙色）
- ✅ 底部状态栏安全区 padding 正常（蓝底 "等待执行..." 不再贴边）
- ⚠️ Monaco 语法高亮在 WebView 中仍不可用（已自动降级，不影响使用）

## 后续可改进

1. **SimpleEditor 断点支持**: 左侧添加行号 gutter，点击切换断点（当前 Monaco 降级后断点功能暂不可用）
2. **多行注释高亮**: 当前只支持 `//` 单行注释，`/* */` 块注释暂未实现
3. **Monaco 升级**: 未来尝试 Monaco 0.50 或更旧版本，或等待 Monaco 修复 WebView 兼容性
