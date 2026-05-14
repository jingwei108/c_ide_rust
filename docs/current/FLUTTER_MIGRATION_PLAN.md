# Cide Flutter 前端迁移计划

> 状态：草案
> 日期：2026-05-13
> 目标：将前端从 .NET MAUI BlazorWebView 迁移至 Flutter，保留 Rust 后端 (`cide_native`) 完全不动。

---

## 目录

1. [方案对比与决策](#1-方案对比与决策)
2. [架构总览](#2-架构总览)
3. [通信层设计](#3-通信层设计)
4. [编辑器替换方案](#4-编辑器替换方案)
5. [模块拆分与工期](#5-模块拆分与工期)
6. [手势系统实现](#6-手势系统实现)
7. [渲染与可视化](#7-渲染与可视化)
8. [风险与回退策略](#8-风险与回退策略)

---

## 1. 方案对比与决策

### 1.1 现有方案的不足

**MAUI + BlazorWebView（当前）**
- WebView 在移动端的 IME/软键盘集成有天然延迟，已有大量 workaround 但仍偶尔闪烁错位
- 复杂拖拽（9元素跨容器交换）在 WebView 内只能靠 JS touch 事件模拟，无原生触觉反馈和拖拽阴影
- iOS 边缘返回手势与 WebView 内部水平滚动/代码选择存在系统级冲突
- APK 体积 ~25-35MB（含 .NET runtime），低端机启动慢
- CSS `visualViewport` 事件有 1-2 帧延迟，SymbolBar 跟随键盘时可能错位

**Tauri + WebView**
- Tauri 在移动端和 MAUI 使用**完全相同的 WebView**（Android System WebView / iOS WKWebView）
- 编辑器体验、手势限制、键盘问题与 MAUI **没有任何区别**
- Tauri 的优势是 Rust 直调（开发体验）和 APK 体积减小，**不能提升用户手指的触感**
- 你要求的 9 元素拖拽在移动端 WebView 中同样难以实现原生级体验

**React Native（已有实验目录）**
- `Cide.Client.ReactNative/` 虽有 UI 框架和 CM6 WebView 集成，但**缺少 Rust 后端桥接**
- RN 的 WebView 手势限制与 MAUI 相同，且 RN 环境配置已被团队判定为"太毒"

### 1.2 为什么选 Flutter

| 维度 | Flutter | MAUI | Tauri |
|------|:-------:|:----:|:-----:|
| 编辑器 IME/软键盘 | ⭐⭐⭐ 原生级 | ⭐⭐ 有 workaround | ⭐⭐ 同 MAUI |
| 复杂拖拽手势 | ⭐⭐⭐ `Draggable` 原生支持 | ⭐ JS 模拟 | ⭐ 同 MAUI |
| 面板拉伸/悬浮球 | ⭐⭐⭐ `GestureDetector` 60fps | ⭐⭐ Blazor touch 事件 | ⭐⭐ 同 MAUI |
| SymbolBar 贴键盘 | ⭐⭐⭐ `MediaQuery` 同步无延迟 | ⭐⭐ CSS 偶尔闪烁 | ⭐⭐ 同 MAUI |
| iOS 边缘手势冲突 | ⭐⭐⭐ 无冲突 | ⭐⭐ 可能冲突 | ⭐⭐ 同 MAUI |
| 启动速度 | ⭐⭐⭐ 快 | ⭐ 慢 | ⭐⭐⭐ 快 |
| APK 体积 | ⭐⭐⭐ ~10MB | ⭐ ~25-35MB | ⭐⭐⭐ ~10-15MB |
| 动画/渲染 | ⭐⭐⭐ Skia 直接 120fps | ⭐⭐ 60fps WebView | ⭐⭐ 同 MAUI |
| 二叉树可视化 | ⭐⭐⭐ `CustomPainter` 原生 | ⭐⭐ Canvas 2D | ⭐⭐ 同 MAUI |
| Rust 后端集成 | ⭐⭐ 需桥接 | ⭐⭐ P/Invoke | ⭐⭐⭐ 直调 |
| 开发成本 | ⭐⭐ 需重写 | ⭐⭐⭐ 已有 | ⭐⭐ 需迁移 |

**核心结论**：Flutter 是唯一能同时在**手势流畅度、键盘集成、渲染性能**三个维度达到原生级的方案。代价是 2.5-3.5 个月重写前端。

---

## 2. 架构总览

### 2.1 迁移后架构

```
Flutter 前端 (Dart)
  ├─ lib/
  │   ├─ main.dart                    # 入口
  │   ├─ app.dart                     # MaterialApp + Theme
  │   ├─ bridge/                      # FRB 生成的绑定 + 手工封装
  │   │   ├─ generated/               # flutter_rust_bridge 自动生成
  │   │   └─ cide_api.dart            # 业务层封装 (~200 行)
  │   ├─ state/                       # 状态管理 (Riverpod)
  │   │   ├─ app_state.dart           # 全局状态
  │   │   ├─ editor_state.dart        # 编辑器状态
  │   │   └─ dock_state.dart          # 9 元素面板状态
  │   ├─ screens/
  │   │   └─ home_screen.dart         # 主页面
  │   ├─ widgets/
  │   │   ├─ editor/                  # 编辑器相关
  │   │   │   ├─ code_editor.dart     # re_editor 封装
  │   │   │   ├─ symbol_bar.dart      # 符号快捷栏
  │   │   │   ├─ template_bar.dart    # 模板栏
  │   │   │   └─ autocomplete.dart    # 智能补全
  │   │   ├─ layout/                  # 布局组件
  │   │   │   ├─ bottom_panel.dart    # 底部面板（可拖拽拉伸）
  │   │   │   ├─ floating_ball.dart   # 悬浮球
  │   │   │   ├─ dock_manager.dart    # 9 元素管理
  │   │   │   └─ toolbar.dart         # 顶部工具栏
  │   │   ├─ panels/                  # 9 个内容面板
  │   │   │   ├─ output_panel.dart
  │   │   │   ├─ diagnostics_panel.dart
  │   │   │   ├─ algorithm_panel.dart
  │   │   │   ├─ callstack_panel.dart
  │   │   │   ├─ watch_panel.dart
  │   │   │   ├─ variables_panel.dart
  │   │   │   ├─ memory_panel.dart
  │   │   │   ├─ array_viz_panel.dart
  │   │   │   ├─ pointer_panel.dart
  │   │   │   └─ knowledge_card_panel.dart
  │   │   ├─ visualization/           # 可视化组件
  │   │   │   ├─ array_bars.dart      # 数组柱状图
  │   │   │   ├─ linked_list_painter.dart  # 链表 CustomPainter
  │   │   │   └─ tree_painter.dart    # 二叉树 CustomPainter
  │   │   └─ common/                  # 通用组件
  │   │       ├─ intro_overlay.dart   # 介绍覆盖层
  │   │       └─ theme_switcher.dart  # 主题切换
  │   └─ models/                      # 数据模型
  │       ├─ diagnostic.dart
  │       ├─ memory_region.dart
  │       ├─ variable.dart
  │       └─ ...
  └─ pubspec.yaml

Rust 后端 (保留不动)
  ├─ native/
  │   ├─ Cargo.toml                   # crate-type 已有 ["cdylib","staticlib","rlib"]
  │   ├─ src/
  │   │   ├─ lib.rs
  │   │   ├─ session.rs               # Session + serde
  │   │   ├─ compiler/                # Lexer/Parser/TypeChecker/BytecodeGen
  │   │   ├─ vm/                      # CideVM
  │   │   ├─ diagnostics/
  │   │   ├─ capi/                    # C API 完全保留
  │   │   └─ flutter_bridge.rs        # 新增：FRB 包装层 (~300 行)
  │   └─ tests/
```

### 2.2 保留与删除

| 保留（不动） | 删除 | 新建 |
|-------------|------|------|
| `native/` 全部 Rust 后端 | `Cide.Client.Maui/` MAUI 项目 | `CideFlutter/` 填充 Flutter 代码 |
| `native/tests/` | `Cide.Client.Shared/` C# 共享库 | `native/src/flutter_bridge.rs` |
| `docs/` 文档 | `Cide.Client.Tests/` xUnit 项目 | Flutter 构建脚本 |
| C API 层（`capi/`）| MAUI 的 `wwwroot/` CSS/JS | `.github/workflows/flutter-ci.yml` |

---

## 3. 通信层设计

### 3.1 选型：flutter_rust_bridge v2

| 方案 | 评估 |
|------|------|
| **dart:ffi + ffigen** | 可复用现有 C API，Rust 零改动。但 Dart 侧代码啰嗦（49 个函数的 count+get 循环），手动管理内存，字符串传递麻烦 |
| **flutter_rust_bridge v2** | ⭐ 推荐。类型安全，自动生成绑定，支持 `Vec<T>` 直接返回，零拷贝大字符串，自动内存管理。只需写 ~300 行 Rust 包装层 |
| **rinf** | 基于消息传递，不适合大量同步查询场景 |
| **irondash** | 过于底层，不必要 |

### 3.2 包装层设计

现有 C API 是 "count + get-by-index" 的 C 风格，FRB 更适合 Rust 风格。新增 `native/src/flutter_bridge.rs`：

```rust
use flutter_rust_bridge::frb;
use std::sync::Mutex;
use lazy_static::lazy_static;
use cide_native::session::Session;

lazy_static! {
    static ref SESSION: Mutex<Session> = Mutex::new(Session::default());
}

#[frb]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub algorithm_matches: Vec<AlgorithmMatch>,
}

#[frb]
pub struct Diagnostic {
    pub line: i32,
    pub column: i32,
    pub error_code: i32,
    pub severity: i32, // 0=error, 1=warning, 2=hint
    pub message: String,
    pub fix_suggestion: String,
}

#[frb]
pub fn compile(source: String) -> CompileResult {
    let mut session = SESSION.lock().unwrap();
    // 调用现有 compiler driver 逻辑
    // ...
    CompileResult { success, diagnostics, algorithm_matches }
}

#[frb]
pub fn run_code() -> RunResult { /* ... */ }

#[frb]
pub fn step_next() -> StepResult { /* ... */ }

#[frb]
pub fn get_variables() -> Vec<VariableSnapshot> { /* ... */ }

#[frb]
pub fn get_memory_regions() -> Vec<MemoryRegion> { /* ... */ }

#[frb]
pub fn get_diagnostics() -> Vec<Diagnostic> { /* ... */ }

#[frb]
pub fn get_callstack() -> Vec<CallStackFrame> { /* ... */ }

#[frb]
pub fn add_breakpoint(line: i32) { /* ... */ }

#[frb]
pub fn clear_breakpoints() { /* ... */ }

#[frb]
pub fn provide_input_line(line: String) { /* ... */ }
```

FRB 自动生成对应的 Dart 类：

```dart
// generated/cide_api.dart (FRB 自动生成)
Future<CompileResult> compile({required String source});
Future<RunResult> runCode();
Future<StepResult> stepNext();
Future<List<VariableSnapshot>> getVariables();
// ...
```

### 3.3 Dart 业务封装层

```dart
// lib/bridge/cide_api.dart
import 'generated/cide_api.dart' as gen;

class CideApi {
  static Future<CompileResult> compile(String source) =>
      gen.compile(source: source);
  
  static Future<RunResult> run() => gen.runCode();
  
  static Future<StepResult> step() => gen.stepNext();
  
  static Future<List<Diagnostic>> getDiagnostics() => gen.getDiagnostics();
  
  static Future<List<MemoryRegion>> getMemoryRegions() => gen.getMemoryRegions();
  
  static Future<List<VariableSnapshot>> getVariables() => gen.getVariables();
  
  static Future<List<CallStackFrame>> getCallStack() => gen.getCallstack();
  
  static Future<void> addBreakpoint(int line) => gen.addBreakpoint(line: line);
  
  static Future<void> clearBreakpoints() => gen.clearBreakpoints();
  
  static Future<void> provideInput(String line) =>
      gen.provideInputLine(line: line);
}
```

---

## 4. 编辑器替换方案

### 4.1 选型：re_editor

| 维度 | re_editor (reqable) | flutter_code_editor (akvelon) |
|------|:-------------------:|:-----------------------------:|
| 底层 | 独立渲染引擎 | Flutter TextField 封装 |
| IME 修复 | ✅ 大量移动端修复 | ⚠️ 依赖 Flutter 框架 |
| 自定义 gutter | ✅ `indicatorBuilder` | ⚠️ `GutterStyle` 有限 |
| 断点标记 | ✅ 可放任意 Widget | ❌ 不支持 |
| 错误/执行行高亮 | ✅ 自定义行装饰 | ⚠️ 仅简单标记 |
| C 高亮 | re_highlight 接入 | highlight 包开箱 |
| 自动补全框架 | ✅ 有接口 | ✅ 简单字典 |
| 性能 | ✅ 大文本优化 | ⚠️ TextField 瓶颈 |

**选型：re_editor**。C 高亮只需接入 `re_highlight` 的 `langC` 模式（约 20 行）。

### 4.2 CM6 功能映射

| CM6 功能（当前） | re_editor 实现方式 | 工作量 |
|-----------------|-------------------|--------|
| C/C++ 语法高亮 | `CodeHighlightTheme(languages: {'c': langC})` | 0.5d |
| 行号 | `DefaultCodeLineNumber` | 0d（自带） |
| 括号匹配 | 需自定义或接入 `DefaultLocalAnalyzer` | 1d |
| 自动闭合括号 | `autocompleteConfig` | 0.5d |
| 代码折叠 | `DefaultCodeChunkIndicator` | 0d（自带） |
| 活动行高亮 | `HighlightFocusLine` | 0.5d |
| 历史记录 | `CodeLineEditingController` 自带 undo/redo | 0d |
| 多选 | `AllowMultipleSelections` | 0.5d |
| **断点 gutter 红点** | 自定义 `indicatorBuilder` + `GestureDetector` | 2-3d |
| **错误行背景高亮** | 自定义 `CodeLineSpanBuilder` 或行装饰 | 2-3d |
| **执行行高亮** | 同上，动态切换 | 1d |
| 滚动到指定行 | `makePositionCenterIfInvisible` | 0.5d |
| 代码模板插入 | `controller.replaceRange()` | 0.5d |
| VS 风格 Enter 格式化 | 监听 `onChanged`，分析上一行缩进 | 2d |
| 主题切换 | `CodeHighlightTheme(theme: oneDarkTheme)` | 0.5d |

**最大风险**：re_editor 的 `indicatorBuilder` 是否支持**点击事件**。如果它只是纯展示 builder，需要 fork 修改添加 `onLineNumberTap` 回调（+3-5 天）。

### 4.3 SymbolBar 贴键盘实现

```dart
@override
Widget build(BuildContext context) {
  final bottomInset = MediaQuery.of(context).viewInsets.bottom;
  final keyboardVisible = bottomInset > 100;

  return Scaffold(
    body: Stack(
      children: [
        Column(
          children: [
            Toolbar(),
            Expanded(child: CodeEditor()),
            if (!keyboardVisible) BottomPanel(),
          ],
        ),
        // SymbolBar 贴在键盘正上方，同步无延迟
        if (keyboardVisible)
          AnimatedPositioned(
            duration: Duration(milliseconds: 100),
            curve: Curves.easeOut,
            bottom: bottomInset,
            left: 0, right: 0, height: 44,
            child: SymbolBar(
              symbols: ['{', '}', '(', ')', '[', ']', '->', '&&', '||', ';'],
              onTap: editorController.insertText,
            ),
          ),
        FloatingBall(),
      ],
    ),
  );
}
```

**优势**：`MediaQuery.viewInsets` 是 Flutter 引擎从系统直接同步的，没有 WebView `visualViewport` 的 1-2 帧延迟，配合 `AnimatedPositioned` 可以做到键盘和 SymbolBar 完全同步的平滑动画。

---

## 5. 模块拆分与工期

> 假设开发者有 Flutter 基础。零基础 +3-4 周学习成本。

### 5.1 代码量统计（需重写）

| 文件/目录 | 行数 | 说明 |
|-----------|------|------|
| `Cide.Client.Shared/Core/*.cs` | ~2,000 | 全部废弃，转 Dart |
| `Cide.Client.Shared/ViewModels/*.cs` | ~200 | 全部废弃 |
| `Cide.Client.Maui/ViewModels/MainViewModel.cs` | 750 | 全部重写 |
| `Home.razor` | 630 | 全部重写 |
| `CodeMirrorEditor.razor` | 148 | 替换为 re_editor |
| `FloatingActionButton.razor` | 258 | 重写 |
| `wwwroot/app.css` | 1,396 | 重写为 Theme + BoxDecoration |
| `wwwroot/js/codemirror-interop.js` | 429 | 重写为 Dart + re_editor API |
| `wwwroot/js/canvas-interop.js` | 108 | 替换为 CustomPainter |
| **合计** | **~5,900 行** | 全部重写 |

### 5.2 工期明细

| Phase | 模块 | 内容 | 最短 | 正常 | 风险 |
|:-----:|------|------|:----:|:----:|:----:|
| 1 | 基础框架 + FFI | Flutter 项目初始化、iOS/Android 配置、Rust .so/.a 编译、FRB 环境配置、端到端验证 | 5d | 8d | 14d |
| 2 | 编辑器核心 | 接入 re_editor、C 高亮、主题、行号、断点标记、错误/执行行高亮、滚动到行 | 6d | 10d | 16d |
| 3 | 编辑器高级 | 代码折叠、括号匹配、Enter 格式化、模板插入、光标移动、undo/redo | 4d | 7d | 12d |
| 4 | 键盘 + SymbolBar | SymbolBar Widget、键盘高度监听、focus/blur 显隐控制 | 2d | 4d | 6d |
| 5 | 手势系统 | 底部面板拖拽拉伸、悬浮球拖拽吸附、9 元素数据模型、拖拽排序、跨容器拖拽、双击删除/收起 | 8d | 14d | 22d |
| 6 | 调试 UI | 底部面板（输出/诊断/算法）、Debug Modal（8 个 Tab）、诊断卡片、知识卡片 | 6d | 10d | 15d |
| 7 | 其他功能 | 智能补全、拍照输入/OCR、介绍覆盖层、主题切换 | 3d | 5d | 8d |
| 8 | 可视化 | 数组柱状图、链表图、内存区域图、二叉树图（CustomPainter） | 4d | 7d | 12d |
| 9 | 测试优化 | Android/iOS 真机测试、性能优化、bug 修复 | 5d | 8d | 14d |
| | **合计** | | **43d (~8.5w)** | **73d (~14.5w)** | **119d (~24w)** |

### 5.3 正常情况工期：约 12-14 周（3-3.5 个月）

---

## 6. 手势系统实现

### 6.1 9 元素数据模型

```dart
enum DockContainer { bottomPanel, floatingBall }

class DockElement {
  final String id;
  final String label;
  final IconData icon;
  DockContainer container;
  int order;
  
  DockElement({
    required this.id, required this.label, required this.icon,
    required this.container, required this.order,
  });
}

// 默认配置
final defaultElements = [
  DockElement(id: 'output', label: '输出', icon: Icons.terminal, container: DockContainer.bottomPanel, order: 0),
  DockElement(id: 'diagnostics', label: '诊断', icon: Icons.warning, container: DockContainer.bottomPanel, order: 1),
  DockElement(id: 'algorithm', label: '算法', icon: Icons.psychology, container: DockContainer.bottomPanel, order: 2),
  DockElement(id: 'knowledge-card', label: '知识卡片', icon: Icons.school, container: DockContainer.floatingBall, order: 0),
  DockElement(id: 'pointer-view', label: '指针视图', icon: Icons.link, container: DockContainer.floatingBall, order: 1),
  DockElement(id: 'array-viz', label: '数组可视化', icon: Icons.bar_chart, container: DockContainer.floatingBall, order: 2),
  DockElement(id: 'memory-region', label: '内存区域', icon: Icons.memory, container: DockContainer.floatingBall, order: 3),
  DockElement(id: 'local-vars', label: '局部变量', icon: Icons.data_object, container: DockContainer.floatingBall, order: 4),
  DockElement(id: 'watch-vars', label: '监视变量', icon: Icons.visibility, container: DockContainer.floatingBall, order: 5),
  DockElement(id: 'call-stack', label: '调用栈', icon: Icons.account_tree, container: DockContainer.floatingBall, order: 6),
];
```

### 6.2 底部面板拖拽拉伸

```dart
GestureDetector(
  onVerticalDragUpdate: (details) {
    setState(() {
      panelHeight = (panelHeight - details.delta.dy)
        .clamp(minPanelHeight, maxPanelHeight);
    });
  },
  child: Container(
    height: 24,
    color: Colors.transparent,
    child: Center(child: Container(width: 40, height: 4, 
      decoration: BoxDecoration(color: Colors.grey, borderRadius: BorderRadius.circular(2)))),
  ),
)
```

### 6.3 悬浮球拖拽 + 贴边吸附

```dart
GestureDetector(
  onPanUpdate: (details) {
    setState(() {
      fabPosition += details.delta;
    });
  },
  onPanEnd: (_) {
    setState(() {
      // 贴边吸附动画
      final screenWidth = MediaQuery.of(context).size.width;
      final targetX = fabPosition.dx < screenWidth / 2 
        ? fabSize * 0.25 
        : screenWidth - fabSize * 0.25;
      fabPosition = Offset(targetX, fabPosition.dy.clamp(minY, maxY));
    });
  },
  child: AnimatedContainer(
    duration: Duration(milliseconds: 200),
    curve: Curves.easeOutBack,
    transform: Matrix4.translationValues(fabPosition.dx - fabSize/2, fabPosition.dy - fabSize/2, 0),
    child: FloatingActionButton(...),
  ),
)
```

### 6.4 跨容器元素拖拽

使用 Flutter 原生 `Draggable` + `DragTarget`：

```dart
Draggable<DockElement>(
  data: element,
  feedback: Material(
    elevation: 4,
    borderRadius: BorderRadius.circular(8),
    child: Container(
      width: 100, height: 40,
      alignment: Alignment.center,
      child: Text(element.label),
    ),
  ),
  childWhenDragging: Opacity(opacity: 0.3, child: ElementCard(element)),
  child: ElementCard(element),
)

DragTarget<DockElement>(
  onAccept: (droppedElement) {
    setState(() {
      // 交换或移动元素
      dockManager.moveElement(droppedElement, targetContainer, targetIndex);
    });
  },
  builder: (context, candidateData, rejectedData) {
    return Container(
      decoration: BoxDecoration(
        border: candidateData.isNotEmpty 
          ? Border.all(color: Colors.blue, width: 2) 
          : null,
      ),
      child: ...,
    );
  },
)
```

**悬浮球上限提示**：当 `floatingBall.elements.length >= 7` 时，`DragTarget` 的 `onWillAccept` 返回 `false`，并显示 SnackBar "悬浮球承载已达上限"。

### 6.5 双击编辑区收起上下栏

```dart
GestureDetector(
  onDoubleTap: () {
    setState(() {
      isFullscreenEditor = !isFullscreenEditor;
    });
  },
  child: AnimatedContainer(
    duration: Duration(milliseconds: 300),
    curve: Curves.easeInOut,
    margin: isFullscreenEditor 
      ? EdgeInsets.zero 
      : EdgeInsets.only(top: toolbarHeight, bottom: bottomPanelHeight),
    child: CodeEditor(),
  ),
)
```

---

## 7. 渲染与可视化

### 7.1 当前可视化实现

| 可视化类型 | 当前实现 | 行数 |
|-----------|---------|------|
| 数组柱状图 | CSS/Blazor (`array-bars` div) | Home.razor 内联 |
| 链表图 | HTML5 Canvas 2D (`canvas-interop.js`) | 64 行 |
| 内存区域图 | HTML5 Canvas 2D (`canvas-interop.js`) | 42 行 |

### 7.2 Flutter 可视化方案

**数组柱状图**：
```dart
Row(
  children: elements.map((e) => Expanded(
    child: AnimatedContainer(
      duration: Duration(milliseconds: 300),
      height: e.heightPercent * maxHeight / 100,
      decoration: BoxDecoration(
        color: e.isHighlighted ? Colors.blue : Colors.grey,
        borderRadius: BorderRadius.circular(4),
      ),
      alignment: Alignment.bottomCenter,
      child: Text(e.value.toString()),
    ),
  )).toList(),
)
```

**链表图（CustomPainter）**：
```dart
class LinkedListPainter extends CustomPainter {
  final List<GraphNode> nodes;
  LinkedListPainter(this.nodes);

  @override
  void paint(Canvas canvas, Size size) {
    final nodePaint = Paint()..color = Colors.grey.shade800;
    final edgePaint = Paint()..color = Colors.grey..strokeWidth = 1.5;
    final textPainter = TextPainter(textDirection: TextDirection.ltr);

    // 画边（带箭头）
    for (var node in nodes) {
      if (node.nextAddr != null) {
        final next = nodes.firstWhere((n) => n.address == node.nextAddr);
        _drawArrow(canvas, edgePaint, node.center, next.topCenter);
      }
    }

    // 画节点
    for (var node in nodes) {
      final rect = RRect.fromRectAndRadius(node.rect, Radius.circular(6));
      canvas.drawRRect(rect, nodePaint..color = node.flashColor ?? node.bgColor);
      canvas.drawRRect(rect, Paint()..color = node.borderColor..style = PaintingStyle.stroke..strokeWidth = 2);
      
      textPainter.text = TextSpan(text: node.label, style: TextStyle(color: Colors.white));
      textPainter.layout();
      textPainter.paint(canvas, node.center - Offset(textPainter.width/2, textPainter.height/2));
    }
  }

  @override
  bool shouldRepaint(covariant LinkedListPainter old) => old.nodes != nodes;
}
```

**二叉树可视化（拓展）**：
```dart
class BinaryTreePainter extends CustomPainter {
  final TreeNode? root;
  final List<int> highlightedNodes; // 遍历高亮路径
  
  @override
  void paint(Canvas canvas, Size size) {
    if (root == null) return;
    _drawNode(canvas, root, size.width / 2, 40, size.width / 4);
  }

  void _drawNode(Canvas canvas, TreeNode node, double x, double y, double dx) {
    final isHighlighted = highlightedNodes.contains(node.id);
    final paint = Paint()
      ..color = isHighlighted ? Colors.blue : Colors.grey.shade700
      ..style = PaintingStyle.fill;
    
    // 画节点圆
    canvas.drawCircle(Offset(x, y), 20, paint);
    
    // 画左右子节点连线
    if (node.left != null) {
      _drawEdge(canvas, Offset(x, y), Offset(x - dx, y + 60));
      _drawNode(canvas, node.left!, x - dx, y + 60, dx / 2);
    }
    if (node.right != null) {
      _drawEdge(canvas, Offset(x, y), Offset(x + dx, y + 60));
      _drawNode(canvas, node.right!, x + dx, y + 60, dx / 2);
    }
    
    // 画文字
    final text = TextPainter(
      text: TextSpan(text: '${node.value}', style: TextStyle(color: Colors.white, fontSize: 14)),
      textDirection: TextDirection.ltr,
    );
    text.layout();
    text.paint(canvas, Offset(x - text.width/2, y - text.height/2));
  }
}
```

### 7.3 动画系统

遍历动画示例（中序遍历高亮）：
```dart
class TreeAnimationController extends ChangeNotifier {
  List<int> _highlightPath = [];
  int _currentIndex = -1;

  Future<void> animateTraversal(List<int> nodeOrder) async {
    _highlightPath = nodeOrder;
    for (int i = 0; i < nodeOrder.length; i++) {
      _currentIndex = i;
      notifyListeners();
      await Future.delayed(Duration(milliseconds: 500));
    }
  }
}
```

**Flutter 渲染优势**：
- `CustomPainter` 直接调用 Skia/Impeller，不经过平台控件层
- `AnimationController` 由 Flutter 引擎 ticker 驱动，保证 60/120fps
- `shouldRepaint` 精确控制重绘区域，避免全屏刷新
- `InteractiveViewer` 内置缩放/平移，适合大图可视化

---

## 8. 风险与回退策略

### 8.1 风险清单

| 风险 | 概率 | 影响 | 缓解措施 |
|------|:----:|:----:|:---------|
| re_editor gutter 不支持点击 | 中 | 高（+3-5d） | POC Day 1 验证；如不支持，fork 修改或降级为长按行号设断点 |
| iOS Rust 静态库编译失败 | 中 | 高（阻塞） | 提前验证 `cargo-lipo` 或 `cargo-xcode` 工具链 |
| FRB 与现有 Rust 类型不兼容 | 低 | 高 | 包装层只做类型转换，不改动核心逻辑 |
| re_editor 自动补全接口不匹配 | 低 | 中 | 自定义 `CompletionProvider` 适配后端符号表 |
| 跨容器拖拽边界情况多 | 中 | 中 | 预留 3-5 天手势调优时间 |
| Flutter 包体积超标 | 低 | 低 | 启用 `split-debug-info` 和 `obfuscate` |

### 8.2 3 天 POC 验证

在投入 3 个月之前，用 **3 个工作日** 验证核心假设：

**Day 1：编辑器验证**
- 新建 Flutter 项目，接入 `re_editor: ^0.8.0`
- 显示一段 C 代码，接入 `re_highlight` C 高亮
- **关键验证**：行号 gutter 能否点击、能否自定义 Widget

**Day 2：FFI 验证**
- 配置 FRB v2 环境
- 写 5 个 Rust 包装函数（compile/run/get_diagnostics）
- Flutter 调用编译并获取诊断信息

**Day 3：手势验证**
- 实现 `Draggable` 方块 + `DragTarget` 区域
- 测试跨容器拖拽流畅度
- iOS 模拟器触摸响应测试

**POC 通过标准**：编辑器能显示 C 代码 + 点击 gutter 有响应 + FFI 编译成功 + 拖拽不掉帧。

### 8.3 回退策略

如果 POC 失败或工期超支：
1. **re_editor 不可用** → 退回 `flutter_code_editor`（基于 TextField，断点标记需改为长按菜单方式）
2. **FRB 配置复杂** → 改用 `dart:ffi + ffigen` 直接复用 C API
3. **Flutter 整体风险过高** → 退回 MAUI，修复现有 bug，9 元素拖拽做 UX 降级（长按菜单代替拖拽）

---

## 9. 关键决策点

| 决策 | 选项 | 推荐 |
|------|------|------|
| 编辑器 | re_editor vs flutter_code_editor | **re_editor**（独立渲染引擎，IME 修复充分） |
| 通信 | FRB v2 vs dart:ffi | **FRB v2**（类型安全，自动生成，长期维护好） |
| SymbolBar | 纯 Flutter vs iOS inputAccessoryView | **纯 Flutter**（跨平台统一，90% 体验） |
| 状态管理 | Riverpod vs Bloc vs Provider | **Riverpod**（类型安全，编译时检查，适合中大型项目） |
| 可视化 | CustomPainter vs 第三方包 | **CustomPainter**（完全可控，性能最好） |

---

## 10. 执行建议

1. **本周**：完成 3 天 POC，验证 re_editor + FRB + 拖拽
2. **POC 通过后**：制定详细迭代计划，每 2 周一个可演示的里程碑
3. **并行进行**：Rust 后端继续维护新功能，Flutter 前端独立开发
4. **过渡期**：MAUI 项目保持维护，Flutter 达到功能对等后再切换主分支
