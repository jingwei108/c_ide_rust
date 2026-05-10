# Android 代码编辑器滚动修复

## 问题描述

在 Android MAUI BlazorWebView 中，CodeMirror 6 编辑器无法正常内部滚动：
- 手指在编辑器区域滑动时，整个 App 页面发生弹性拉伸（overscroll/elastic stretch）
- 代码内容无法滚动查看，编辑功能异常

## 根因分析

1. **高度塌陷**：`.cm-editor` 默认的 `position: absolute` 在 WebView 中导致其包裹层高度坍缩为 0，Android WebView 无法识别这是一个可滚动内嵌区域，遂将手势交由页面级处理。
2. **Scroller 高度被错误压缩**：初期修复脚本误将绝对定位的 `.cm-gutters` 高度从 pane 高度中扣除（`paneH - gutterHeight`）。由于 gutters 实际不占文档流空间，这导致 `.cm-scroller` 的可视高度被压到 120px，下方出现大面积空白背景，仅顶部小窗口能显示代码。

## 修复方案

### 1. 运行时强制显式高度（`index.html`）

通过 JS 在 `load / resize` 以及定时器中：
- 将 `.cm-editor` 设为 `position: relative` 并显式指定 `height = pane.clientHeight`
- 将 `.cm-scroller` 的 `height / max-height` 直接设为 pane 高度（**不减去 gutter 高度**）
- 同时设置 `overflow: auto`、`-webkit-overflow-scrolling: touch`、`touch-action: pan-x pan-y`
- 使用 `MutationObserver` 监听 CodeMirror 对 `style` 属性的修改，自动重新应用

### 2. CSS 层（`app.css`）

- `.cide-app`：`overflow-y: auto` + `-webkit-overflow-scrolling: touch` + `touch-action: pan-y`
- `.editor-pane`：`touch-action: pan-x pan-y`
- `.cm-scroller`：`overflow: auto !important` + `-webkit-overflow-scrolling: touch !important` + `touch-action: pan-x pan-y !important`
- `.template-scroll`：水平模板栏独立 `touch-action: pan-x`

### 3. Razor 层（`Home.razor`）

- 移除 `.editor-pane` 上的 `@ontouchstart/move/end` Blazor 事件绑定，避免拦截本应传递给 CodeMirror 的原生触摸事件。

## 验证数据

修复后 on-device 调试输出：
```
paneH=450 | editorOH=450 pos=relative h=450px |
scrollH=995 clientH=450 offsetH=450 ov=auto
```
- `clientH=450` 表示 scroller 可视区域占满 editor-pane，无塌陷/无空白。
- 内容 `scrollH=995 > clientH=450`，内部滚动正常。

## 涉及文件

- `Cide.Client.Maui/wwwroot/index.html`
- `Cide.Client.Maui/wwwroot/app.css`
- `Cide.Client.Maui/Components/Pages/Home.razor`

## 日期

2026-05-10
