# Cide 编辑器重构架构方案

> 状态：草案（含缺陷修正 v6）  
> 日期：2026-05-18  
> 背景：从 CM6 WebView 到 re_editor 的两次迁移均产生了深层技术债，现决定打造面向 Cide 场景的专用编辑器内核。

---

## 目录

1. [历史背景：两次妥协](#1-历史背景两次妥协)
2. [当前痛点](#2-当前痛点)
3. [设计哲学](#3-设计哲学)
4. [产品视角：学生用户需要快捷键吗](#4-产品视角学生用户需要快捷键吗)
5. [总体架构](#5-总体架构)
6. [Gesture Proxy 模式详解](#6-gesture-proxy-模式详解)
7. [CideDocument 与 Undo/Redo](#7-cidedocument-与-undoredo)
8. [输入分流设计](#8-输入分流设计)
9. [快捷键设计（极简）](#9-快捷键设计极简)
10. [滚动架构](#10-滚动架构)
11. [TextInputConnection 生命周期](#11-textinputconnection-生命周期)
12. [性能优化：换行符索引](#12-性能优化换行符索引)
13. [四大验证项](#13-四大验证项)
14. [实施路线图](#14-实施路线图)
15. [已知缺陷与修正记录](#15-已知缺陷与修正记录)
16. [风险与回退](#16-风险与回退)
17. [塞级问题备忘录](#17-塞级问题备忘录)

---

## 1. 历史背景：两次妥协

| 阶段 | 编辑器 | 获得的能力 | 付出的代价 | 根本缺陷 |
|------|--------|-----------|-----------|----------|
| **CM6 时代** | WebView CodeMirror 6 | 完整的文档模型、Decoration 系统、Gutter 插件生态 | WebView 键盘延迟、手势冲突、APK 臃肿 | **容器不对**（WebView 不适合移动端 IDE） |
| **re_editor 时代** | Flutter 原生 CustomPainter | 原生键盘、原生手势、120fps 渲染 | 无文档模型、无 Decoration、无开放 API | **内核不对**（文本框不是代码画布） |

两次迁移都是**用一个维度的妥协换另一个维度的能力**。从 git 历史可见，`editor_panel.dart` 在 Flutter 迁移后经历了 **11 次功能入侵式修改**，平均每 2-3 天就要为编辑器的 API 限制打补丁。

---

## 2. 当前痛点

### 2.1 深层 Hack：5 处 `as dynamic`

`editor_panel.dart` 中通过 `as dynamic` 访问 `re_editor` 私有 API：

```dart
final internalKey = (codeEditorState as dynamic)._editorKey as GlobalKey?;
final range = (renderBox as dynamic).selectWord(position: globalPosition);
final sel = (renderBox as dynamic).setPositionAt(position: globalPosition);
```

这些是编译期无法检查的地雷，一旦 `re_editor` 内部改名即运行时崩溃。

### 2.2 状态散落于 Widget

编辑器相关状态散落在 `EditorPanelState` 中：

```dart
int _currentHighlightLine = 0;
List<AccessedVar> _currentAccessedVars = [];
Set<int> _currentTutorialLines = {};
Map<int, int> diagMap = {};
Set<int> breakpoints = {};
```

这些状态本应在**文档模型**中管理，却被迫耦合在 UI 层。

### 2.3 re_editor 大包大揽

`CodeEditor` 在 `initState` 中**无条件创建**：
- `CodeFindController`（查找替换，Cide 未使用）
- `CodeChunkController`（代码折叠，启动 isolate 分析，Cide 未使用）
- `_CodeFloatingCursorController`（iOS 浮动光标）
- `SelectionToolbarController`（系统选择工具栏，Cide 已自建长按菜单）

---

## 3. 设计哲学

### 3.1 核心原则

> **能交给 Flutter 框架的，就交给 Flutter 框架。我们只接管渲染和 Cide 特有的逻辑。**

这与 Cide 已确立的设计哲学一致：
- 物理键盘 → 交给系统
- 系统输入法 → 交给 `TextInputConnection`
- 触屏手势 → 交给 `EditableText`
- Cide 只负责：**自绘键盘、文档模型、图层渲染**

### 3.2 从"文本框"到"代码画布"

| 维度 | 文本框思维（re_editor） | 代码画布思维（CideCanvas） |
|------|----------------------|--------------------------|
| 核心数据 | `String` / `CodeLines` | `CideDocument` = 文本 + AST + 运行时状态 + 教程状态 |
| 渲染 | 单 `TextSpan` → `TextPainter` | 多图层叠加（Text + Syntax + Diagnostic + Runtime + Tutorial） |
| 扩展 | `spanBuilder` 回调改 `TextSpan` | 注册 `EditorLayer` 插件，接收文档事件，独立绘制 |
| 状态同步 | `forceRepaint()` 全量重绘 | 精确到行/字符的增量更新 |
| 坐标查询 | 字符串匹配遍历 `TextSpan.children` | 行级 `TextPainter.getOffsetForCaret` + 缓存 |

---

## 4. 产品视角：学生用户需要快捷键吗？

### 4.1 当前快捷键清单

```dart
F5       → 运行
F10      → 单步
F9       → 切换断点
Shift+F5 → 停止
```

就 4 个，全部是**桌面 IDE（VS Code/Visual Studio）的 F 键映射**。

### 4.2 学生真的有快捷键意识吗？

**没有。而且几乎不可能有。**

- 编程初学者的第一接触点是按钮，不是快捷键。他们连 `printf` 语法都记不住，不可能记住 F5=运行。
- 移动端的物理限制：绝大多数平板键盘套**没有 F1-F12 键行**。F5 在物理上就不存在。
- Cide 已有完整的 UI 替代方案：悬浮球（运行）、执行控制面板（单步）、点击行号 gutter（断点）。

### 4.3 移动端主交互是"可见即可点"

| 操作 | 当前 UI 替代方案 | 快捷键价值 |
|------|-----------------|-----------|
| 运行 | 悬浮球（发光 orb，屏幕中央偏右） | **零**。学生一眼就能看到，点一下就行 |
| 单步 | 执行控制面板底部按钮 | **零**。按钮上有 ▶️ 和 ⏸️ 图标 |
| 断点 | 点击行号左侧 gutter | **零**。比按 F9 更直观 |
| 停止 | 悬浮球菜单/执行面板 | **零** |
| 撤销 | 自绘键盘上的 ↩️ 按钮 | **零** |
| 复制/粘贴 | 长按菜单 | 低 |

### 4.4 架构启示

由于 Cide 的核心战场是**移动端学生用户**，快捷键不是核心体验：
- **移动端**：完全不需要 F 键快捷键。自绘键盘 + 悬浮球 + 底部面板已覆盖所有操作。
- **桌面端**：`Shortcuts` 可以保留 F5/F9/F10 作为锦上添花，但不需要复杂的拦截逻辑。
- **编辑快捷键**（Ctrl+Z/A/C/V、方向键等）：全部由 `EditableText` 内置处理，Cide 不拦截。

这意味着 **Gesture Proxy 架构中不需要 `FocusNode.onKeyEvent` 拦截任何快捷键**。`EditableText` 和 Cide 的 `Shortcuts` 天然不冲突——F 键不是文本编辑键，`EditableText` 会直接冒泡。

---

## 5. 总体架构

```
CideEditor Widget
└── SingleChildScrollView(          ← 统一滚动控制器
      controller: _scrollController,
      child: SizedBox(
        height: _totalContentHeight,
        child: Stack(
          children: [
            Positioned.fill
              child: Listener(        ← 记录点击坐标
                onPointerDown: (e) => _lastPointerPosition = e.position,
                child: EditableText(  ← Gesture Proxy + IME Proxy
                  controller: _proxyController,
                  focusNode: _focusNode,
                  style: TextStyle(color: Colors.transparent),
                  cursorColor: Colors.transparent,
                  backgroundCursorColor: Colors.transparent,
                  cursorOpacityAnimates: false,
                  selectionColor: Colors.transparent,
                  scrollPadding: EdgeInsets.zero,  // 禁止自行滚动
                  maxLines: null,
                  autocorrect: false,
                  enableSuggestions: false,
                ),
              ),
            Positioned.fill
              child: CustomPaint(
                painter: CideEditorPainter(
                  document: _document,
                  scrollOffset: _scrollController.offset,
                  layers: [...],
                ),
              ),
          ],
        ),
      ),
    )
```

---

## 6. Gesture Proxy 模式详解

### 6.1 核心思想

`EditableText` 不做任何可见渲染，只作为**手势代理**和**IME 代理**：
- 接收所有系统手势（单击、双击、长按、拖拽选择、手柄拖拽、放大镜）
- 接收系统 IME（桌面物理键盘、移动端系统键盘）
- 通过 `_proxyController` 暴露 `text` 和 `selection` 变化

`CustomPaint` 负责所有可见渲染，以 `_document` 为唯一数据源。

### 6.2 点击坐标获取

`EditableText.onTap` 是 `VoidCallback?`，**不提供坐标**。用 `Listener` 包裹 `EditableText`：

```dart
Offset? _lastPointerPosition;

Listener(
  onPointerDown: (event) => _lastPointerPosition = event.position,
  child: EditableText(
    onTap: () {
      if (_lastPointerPosition != null) {
        _handleTap(_lastPointerPosition!);
      }
    },
  ),
)
```

**事件顺序**：`onPointerDown`（记录坐标）→ `EditableText` 内部处理 → `onTap`（读取坐标）。`Listener` 响应原始 `PointerEvent`，不受 `EditableText` 内部 `GestureDetector` 竞技场影响。

### 6.3 双向同步（增量版）

```dart
class CideEditorState extends State<CideEditor> {
  late final TextEditingController _proxyController;
  late final CideDocument _document;
  bool _syncing = false;

  void _onProxyChanged() {
    if (_syncing) return;
    _syncing = true;

    final proxy = _proxyController.value;
    final oldText = _document.text;

    if (proxy.text != oldText) {
      final diff = _computeDiff(oldText, proxy.text);
      if (diff != null) {
        _document.applyEdit(diff);
      } else {
        _document.setText(proxy.text);
      }
    }

    _document.updateSelection(
      baseLine: _document.offsetToLine(proxy.selection.baseOffset),
      baseCol: _document.offsetToCol(proxy.selection.baseOffset),
      extentLine: _document.offsetToLine(proxy.selection.extentOffset),
      extentCol: _document.offsetToCol(proxy.selection.extentOffset),
    );

    _document.updateComposing(proxy.composing);
    _syncing = false;
  }

  void _onDocumentChanged() {
    if (_syncing) return;
    _syncing = true;

    _proxyController.value = TextEditingValue(
      text: _document.text,
      selection: _toTextSelection(_document.selection),
      composing: _document.composing,
    );

    _inputConnection?.setEditingState(_proxyController.value);
    _syncing = false;
  }
}
```

### 6.4 增量 diff 算法

```dart
EditOp? _computeDiff(String oldText, String newText) {
  int commonPrefix = 0;
  while (commonPrefix < oldText.length &&
         commonPrefix < newText.length &&
         oldText[commonPrefix] == newText[commonPrefix]) {
    commonPrefix++;
  }

  int commonSuffix = 0;
  while (commonSuffix < oldText.length - commonPrefix &&
         commonSuffix < newText.length - commonPrefix &&
         oldText[oldText.length - 1 - commonSuffix] ==
         newText[newText.length - 1 - commonSuffix]) {
    commonSuffix++;
  }

  final start = commonPrefix;
  final oldEnd = oldText.length - commonSuffix;
  final newEnd = newText.length - commonSuffix;

  return EditOp(
    startOffset: start,
    oldText: oldText.substring(start, oldEnd),
    newText: newText.substring(start, newEnd),
  );
}
```

---

## 7. CideDocument 与 Undo/Redo

### 7.1 核心策略：单输入源，禁止双键盘共存

Cide 明确设置**不允许双输入源同时存在**。有自绘键盘活跃时，系统输入法关闭；使用系统输入法时，自绘键盘隐藏。

**阻止系统 IME 的正确方式是断开 `TextInputConnection`，而不是 `readOnly = true`。**

`readOnly = true` 会禁用文本编辑，但**不会影响选择手柄**（除非同时设置 `enableInteractiveSelection = false`）。然而，为了语义清晰和避免意外，自绘键盘模式下不应修改 `EditableText` 的 `readOnly` 属性，而是通过断开 `TextInputConnection` 来阻止系统输入法输入。

```dart
void showCustomKeyboard() {
  _isSystemKeyboardActive = false;
  _detachInputConnection();  // 断开 TextInputConnection，阻止系统 IME
  _hideCustomKeyboardOverlay(); // 隐藏系统键盘 UI
}

void showSystemKeyboard() {
  _isSystemKeyboardActive = true;
  _hideCustomKeyboardOverlay(); // 隐藏自绘键盘
  if (_focusNode.hasFocus) {
    _attachInputConnection();  // 连接 TextInputConnection，接收系统 IME
  }
}
```

### 7.2 分设备类型的 Undo 路径

| 设备类型 | 输入源 | Undo 机制 |
|---------|--------|----------|
| 手机（无物理键盘） | 自绘键盘 | `_document._undoStack`，Undo 按钮调用 `_document.undo()` |
| 平板（无键盘） | 自绘键盘 | 同上 |
| 平板 + 键盘/桌面 | 系统物理键盘 | `EditableText` 内置 `UndoHistoryController`，`Ctrl+Z` 自动处理 |

切换键盘时清空另一方栈：

```dart
void switchToSystemKeyboard() {
  _document.clearUndoStack();
  showSystemKeyboard();
}
```

### 7.3 移动端 UndoManager

```dart
class EditOp {
  final int startOffset;
  final String oldText;
  final String newText;

  EditOp({required this.startOffset, required this.oldText, required this.newText});
  EditOp inverse() => EditOp(startOffset: startOffset, oldText: newText, newText: oldText);
}

class CideDocument {
  final List<EditOp> _undoStack = [];
  final List<EditOp> _redoStack = [];
  static const int maxHistory = 200;

  void applyEdit(EditOp op) {
    _apply(op);
    _undoStack.add(op);
    if (_undoStack.length > maxHistory) _undoStack.removeAt(0);
    _redoStack.clear();
    notifyListeners();
  }

  void undo() {
    if (_undoStack.isEmpty) return;
    final op = _undoStack.removeLast().inverse();
    _apply(op);
    _redoStack.add(op.inverse());
    notifyListeners();
  }

  void clearUndoStack() {
    _undoStack.clear();
    _redoStack.clear();
  }
}
```

---

## 8. 输入分流设计

### 8.1 三条输入路径（互斥）

```
设备检测
├── 无物理键盘（手机/纯平板）
│   └── 强制使用 Cide CustomKeyboard
│   └── TextInputConnection 断开（阻止系统 IME）
│   └── 自绘键盘直接操作 _document
│
└── 有物理键盘（平板+键盘/桌面）
    └── 强制使用系统输入法
    └── 自绘键盘隐藏
    └── TextInputConnection 连接（接收系统 IME）
```

### 8.2 自绘键盘接入

```dart
void insertText(String text) {
  final op = _document.createInsertOp(text);
  _document.applyEdit(op);
  _syncToProxy();
}

void _syncToProxy() {
  _proxyController.value = TextEditingValue(
    text: _document.text,
    selection: _toTextSelection(_document.selection),
    composing: _document.composing,
  );
}
```

---

## 9. 快捷键设计（极简）

**Cide 不需要拦截任何编辑快捷键。**

`EditableText` 内置：
- `Ctrl+Z` / `Ctrl+Y` → 撤销/重做（桌面端）
- `Ctrl+A` / `Ctrl+C` / `Ctrl+V` / `Ctrl+X` → 全选/复制/粘贴/剪切
- 方向键、退格、Delete → 基础编辑

Cide 特有的 F 键（桌面端锦上添花）：

```dart
final shortcuts = <ShortcutActivator, Intent>{
  const SingleActivator(LogicalKeyboardKey.f5): const _RunIntent(),
  const SingleActivator(LogicalKeyboardKey.f10): const _StepIntent(),
  const SingleActivator(LogicalKeyboardKey.f9): const _ToggleBreakpointIntent(),
  const SingleActivator(LogicalKeyboardKey.f5, shift: true): const _StopIntent(),
};
```

F 键与 `EditableText` 天然不冲突，不需要 `FocusNode.onKeyEvent` 拦截。

---

## 10. 滚动架构

### 10.1 问题：滚动是 Gesture Proxy 模式中最严重的缺失

如果 `EditableText` 和 `CustomPaint` 的滚动不同步，会出现：
- 文本滚走了，但 `RuntimeLayer` 的执行行底色还留在原地
- 选择手柄跟随 `EditableText` 滚动，但 `CustomPaint` 文本在另一个位置

### 10.2 方案：外层 SingleChildScrollView 统一控制

`EditableText` 和 `CustomPaint` 都不自行滚动，完全由外层的 `SingleChildScrollView` 控制：

```dart
SingleChildScrollView(
  controller: _scrollController,
  child: SizedBox(
    height: _totalContentHeight,
    child: Stack(
      children: [
        Positioned.fill(
          child: Listener(
            onPointerDown: (e) => _lastPointerPosition = e.position,
            child: EditableText(
              scrollPadding: EdgeInsets.zero,  // 禁止 EditableText 自行滚动
              maxLines: null,                  // 扩展至内容高度
              // ... 其他配置
            ),
          ),
        ),
        Positioned.fill(
          child: CustomPaint(
            painter: CideEditorPainter(
              document: _document,
              viewport: Rect.fromLTWH(
                0,
                _scrollController.offset,
                constraints.maxWidth,
                constraints.maxHeight,
              ),
              layers: [...],
            ),
          ),
        ),
      ],
    ),
  ),
)
```

**关键点**：
- `EditableText.scrollPadding = EdgeInsets.zero`：禁止它自己处理滚动偏移
- `EditableText.maxLines = null`：让它扩展至完整内容高度
- `CustomPaint` 的 `painter` 接收 `viewport`（当前可见区域），只绘制可见行
- 两者在同一个 `SingleChildScrollView` 中，滚动天然同步

### 10.3 内容高度计算

```dart
double get _totalContentHeight {
  return document.lines.fold(0.0, (sum, line) => sum + line.height);
}
```

### 10.4 CustomPaint 只绘制可见行

```dart
class CideEditorPainter extends CustomPainter {
  final CideDocument document;
  final Rect viewport;

  @override
  void paint(Canvas canvas, Size size) {
    for (int i = 0; i < document.lines.length; i++) {
      final line = document.lines[i];
      final lineTop = i * line.height;
      final lineBottom = lineTop + line.height;

      // 只绘制可见行
      if (lineBottom < viewport.top || lineTop > viewport.bottom) continue;

      final lineRect = Rect.fromLTWH(0, lineTop, size.width, line.height);

      // 绘制各图层
      for (final layer in layers) {
        layer.paint(canvas, lineRect, line, document);
      }
    }
  }
}
```

---

## 11. TextInputConnection 生命周期

### 11.1 状态机

```
[初始化]
  └── _inputConnection = null

[编辑器获得焦点 onFocus]
  └── 如果是系统键盘模式：
        _inputConnection = TextInput.attach(this, config)
        _inputConnection.show()
      如果是自绘键盘模式：
        不连接（保持 null）

[编辑器失去焦点 onBlur]
  └── _inputConnection?.close()
        _inputConnection = null

[切换到系统键盘]
  └── 如果 _inputConnection == null：
        _inputConnection = TextInput.attach(this, config)
      _inputConnection.show()

[切换到自绘键盘]
  └── _inputConnection?.close()
        _inputConnection = null
```

### 11.2 实现

```dart
class CideEditorState extends State<CideEditor> implements TextInputClient {
  TextInputConnection? _inputConnection;
  bool _isSystemKeyboardActive = false;

  @override
  void initState() {
    super.initState();
    _focusNode.addListener(_onFocusChanged);
  }

  void _onFocusChanged() {
    if (_focusNode.hasFocus) {
      if (_isSystemKeyboardActive) {
        _attachInputConnection();
      }
    } else {
      _detachInputConnection();
    }
  }

  void _attachInputConnection() {
    if (_inputConnection != null) return;
    _inputConnection = TextInput.attach(
      this,
      TextInputConfiguration(
        inputType: TextInputType.multiline,
        inputAction: TextInputAction.newline,
        autocorrect: false,
        enableSuggestions: false,
        enableIMEPersonalizedLearning: false,
      ),
    );
    _inputConnection!.show();
  }

  void _detachInputConnection() {
    _inputConnection?.close();
    _inputConnection = null;
  }

  void showSystemKeyboard() {
    _isSystemKeyboardActive = true;
    if (_focusNode.hasFocus) _attachInputConnection();
  }

  void showCustomKeyboard() {
    _isSystemKeyboardActive = false;
    _detachInputConnection();
  }

  @override
  void dispose() {
    _detachInputConnection();
    _focusNode.removeListener(_onFocusChanged);
    super.dispose();
  }
}
```

---

## 12. 性能优化：换行符索引

### 12.1 问题：`_lineIndexFromOffset` 是 O(n)

每次 `_onProxyChanged` 都需要把全局 offset 转换为 line+col。如果逐行累加计算，500 行代码就是 500 次操作。

### 12.2 方案：缓存行起始偏移 + 二分查找

```dart
class CideDocument {
  String _text = '';
  List<int> _lineStartOffsets = [0];

  void setText(String text) {
    _text = text;
    _rebuildLineOffsets();
    notifyListeners();
  }

  void _rebuildLineOffsets() {
    _lineStartOffsets = [0];
    for (int i = 0; i < _text.length; i++) {
      if (_text[i] == '\n') _lineStartOffsets.add(i + 1);
    }
  }

  /// O(log n) 二分查找
  int offsetToLine(int offset) {
    int lo = 0, hi = _lineStartOffsets.length - 1;
    while (lo < hi) {
      final mid = (lo + hi + 1) ~/ 2;
      if (_lineStartOffsets[mid] <= offset) lo = mid;
      else hi = mid - 1;
    }
    return lo;
  }

  int offsetToCol(int offset) {
    final line = offsetToLine(offset);
    return offset - _lineStartOffsets[line];
  }
}
```

### 12.3 增量更新时避免全量重建

如果 `applyEdit` 是增量操作（已知 startOffset），可以只更新受影响的行：

```dart
void applyEdit(EditOp op) {
  // 替换文本
  _text = _text.substring(0, op.startOffset) +
         op.newText +
         _text.substring(op.startOffset + op.oldText.length);

  // 从 startOffset 所在行开始重建，而非全量重建
  final startLine = offsetToLine(op.startOffset);
  _rebuildLineOffsetsFrom(startLine);

  notifyListeners();
}
```

---

## 13. 四大验证项

### 验证 1：EditableText 透明性

```dart
EditableText(
  style: TextStyle(color: Colors.transparent),
  cursorColor: Colors.transparent,
  cursorOpacityAnimates: false,
  backgroundCursorColor: Colors.transparent,
  selectionColor: Colors.transparent,
)
```

**验证方法**：肉眼确认完全看不到系统渲染内容。

### 验证 2：Composing Range（含跨行）

中文输入法长句子 composing 可能跨多行。基于全局 offset 转换：

```dart
void _drawComposingLayer(Canvas canvas) {
  final composing = document.composing;
  if (composing.isCollapsed) return;

  final startPos = document.offsetToPosition(composing.start);
  final endPos = document.offsetToPosition(composing.end);

  for (int line = startPos.line; line <= endPos.line; line++) {
    final layout = document.lineLayout(line);
    final start = line == startPos.line ? startPos.col : 0;
    final end = line == endPos.line ? endPos.col : layout.text.length;

    final boxes = layout.painter.getBoxesForSelection(
      TextSelection(baseOffset: start, extentOffset: end),
    );
    for (final box in boxes) {
      _drawDashedLine(canvas, box.toRect().bottom);
    }
  }
}
```

**验证方法**：中文输入法输入跨两行长句子，确认下划线位置正确。

### 验证 3：Listener + EditableText 事件竞争

**验证方法**：在最简 Demo 中点击编辑器，确认：
1. `Listener.onPointerDown` 触发并记录坐标
2. `EditableText.onTap` 触发并读取到正确坐标
3. 选择、长按、拖拽等手势正常工作

### 验证 4：滚动同步

**验证方法**：
1. 输入 200 行代码
2. 选择一段跨多行的文本
3. 滚动编辑器
4. 确认：文本、选区背景、选择手柄、光标同步滚动，无漂移

### 验证 5：性能 Profile

**验证方法**：500 行代码，对比"纯 `EditableText`"和"Gesture Proxy + `CustomPaint`"的帧率。

---

## 14. 实施路线图

### Phase 0：独立验证 POC（3 天）

```dart
class Phase0Demo extends StatefulWidget {
  @override
  State createState() => _Phase0DemoState();
}

class _Phase0DemoState extends State<Phase0Demo> {
  final _proxy = TextEditingController(text: 'Hello World\n第二行中文测试');
  final _focusNode = FocusNode();
  final _scrollController = ScrollController();

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      controller: _scrollController,
      child: SizedBox(
        height: 2000, // 模拟长文本
        child: Stack(
          children: [
            Positioned.fill(
              child: Listener(
                onPointerDown: (e) => _lastPointer = e.position,
                child: EditableText(
                  controller: _proxy,
                  focusNode: _focusNode,
                  style: TextStyle(color: Colors.transparent, fontSize: 24),
                  cursorColor: Colors.transparent,
                  backgroundCursorColor: Colors.transparent,
                  selectionColor: Colors.transparent,
                  scrollPadding: EdgeInsets.zero,
                  maxLines: null,
                  autocorrect: false,
                  enableSuggestions: false,
                ),
              ),
            ),
            Positioned.fill(
              child: CustomPaint(
                painter: _SimpleTextPainter(
                  text: _proxy.text,
                  selection: _proxy.selection,
                  scrollOffset: _scrollController.offset,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
```

**验证清单**：
1. `EditableText` 透明化，无可见残留
2. `CustomPaint` 叠加绘制文本 + 选区背景
3. 中文输入法 composing 下划线自绘（含跨行）
4. `Listener.onPointerDown` + `EditableText.onTap` 坐标正确
5. 滚动时文本、选区、手柄同步无漂移
6. 500 行代码性能 Profile

### Phase 1：Gesture Proxy 最小接入（1 周）

- 不引入 `CodeLineEditingController`
- 直接用 `EditableText` 的 `_proxyController` 作为唯一文本源
- 接入外层 `SingleChildScrollView` 统一滚动
- 验证双向同步、自绘键盘、滚动同步

### Phase 2：文档模型 + 移动端 UndoManager（1 周）

- 设计 `CideDocument`，内部用 `line+col`，对外提供 `offset` 转换
- 缓存 `_lineStartOffsets`，二分查找 O(log n)
- 迁移散落状态
- 实现 `UndoManager`
- **Accessibility**：给 `CustomPaint` 包裹 `Semantics`

```dart
Semantics(
  textField: true,
  label: 'Cide Editor',
  value: _document.text,
  textSelection: _toTextSelection(_document.selection),
  child: CustomPaint(...),
)
```

### Phase 3：图层系统（2 周）

实现 `EditorLayer` 接口：
- `DiagnosticLayer`
- `RuntimeLayer`
- `TutorialLayer`
- `ComposingLayer`

### Phase 4：Gutter 插件化 + 内核精简（2 周）

- `GutterColumn` 插件系统
- 删除 `re_editor` 无用模块
- 废除 `part of` 模式

### Phase 5：移除 re_editor（1 天）

---

## 15. 已知缺陷与修正记录

| 编号 | 问题 | 状态 | 解决方案 |
|------|------|------|----------|
| 1 | Undo/Redo 完全缺失 | ✅ | 单输入源策略，移动端自研 `UndoManager`，桌面端依赖 `EditableText` |
| 2 | Phase 顺序错误 | ✅ | Phase 1 先做 Gesture Proxy，Phase 2 再做文档模型 |
| 3 | Shortcuts 与 EditableText 冲突 | ✅ | 不需要拦截编辑快捷键，F 键天然不冲突 |
| 4 | 自绘键盘绕过 TextInputConnection | ✅ | 自绘键盘输入后 `setEditingState()` |
| 5 | EditableText 全文本 layout 开销 | ✅ | Phase 0 Profile 验证 |
| 6 | Phase 0 Demo 未独立 | ✅ | 完全独立于 `re_editor` |
| 7 | 全量同步 O(n) | ✅ | 前缀/后缀比较 `_computeDiff()` |
| 8 | `EditableText.onTap` 无坐标 | ✅ | `Listener.onPointerDown` 记录 |
| 9 | 双键盘 Undo 数据孤岛 | ✅ | 单输入源策略，切换键盘清空 undo 栈 |
| 10 | Phase 1 引入 `CodeLineEditingController` | ✅ | 直接用 `_proxyController` |
| 11 | Accessibility 缺失 | ✅ | `Semantics` widget |
| 12 | 滚动架构缺失 | ✅ | `SingleChildScrollView` 统一控制 |
| 13 | `_inputConnection` 生命周期未定义 | ✅ | 焦点变化 + 键盘切换时连接/断开 |
| 14 | 换行符索引 O(n) | ✅ | 缓存 + 二分查找 |
| 15 | Listener 事件竞争 | ✅ | `Listener` 响应原始 `PointerEvent` |
| 16 | `showCustomKeyboard` 实现不一致 | ✅ | 统一为断开 `_inputConnection`，不设置 `readOnly` |

---

## 16. 风险与回退

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| POC 验证 4 失败（手柄偏移） | 中 | 高 | **回退**：保留 `re_editor` 渲染 |
| 中文输入法 composing 对不齐 | 低 | 中 | Phase 0 验证 |
| 自绘键盘与 `EditableText` 状态竞争 | 中 | 中 | `_syncing` 标志 + 单输入源 |
| 性能退化（两层 TextPainter） | 低 | 中 | Phase 0 Profile + 只绘制可见行 |
| 滚动不同步 | 中 | **极高** | Phase 0 必须验证滚动同步 |
| 重构周期过长 | 中 | 中 | 分 Phase 实施 |

---

## 17. 塞级问题备忘录

> "塞级问题" = 必须解决才能进入 Phase 0 的阻塞性问题。

| 问题 | 状态 | 解决方案 |
|------|------|----------|
| `EditableText.onTap` 无坐标 | ✅ | `Listener.onPointerDown` 记录 |
| 双键盘 Undo 数据孤岛 | ✅ | 单输入源策略 |
| Phase 1 引入 `CodeLineEditingController` | ✅ | 直接用 `_proxyController` |
| `_computeDiff` 未实现 | ✅ | 前缀/后缀比较 |
| Composing 跨行 | ✅ | 全局 offset 转换 |
| Accessibility 缺失 | ✅ | `Semantics` widget |
| 滚动架构缺失 | ✅ | `SingleChildScrollView` 统一控制 |
| `_inputConnection` 生命周期 | ✅ | 焦点变化 + 键盘切换时连接/断开 |
| 换行符索引性能 | ✅ | 缓存 + 二分查找 |
| Listener 事件竞争 | ✅ | `Listener` 响应原始 `PointerEvent` |
| `showCustomKeyboard` 不一致 | ✅ | 统一断开 `_inputConnection`，不碰 `readOnly` |

---

> **核心结论**：Gesture Proxy 模式是务实的中间路线——保留 Flutter 成熟的手势、IME 和滚动能力，只接管渲染和 IDE 语义。
