# 自定义虚拟键盘实现记录

> 为 Cide Flutter 移动端实现了一套完全自定义的代码输入键盘，替代系统虚拟键盘。

---

## 问题背景

### 1. 系统键盘导致快捷栏延迟
- 特殊符号快捷栏使用 `MediaQuery.of(context).viewInsets.bottom` 检测键盘高度
- 系统键盘弹出有动画延迟（约 200-300ms），快捷栏在键盘完全弹出后才显示
- 视觉效果割裂，用户体验差

### 2. 底部面板被键盘顶起
- 系统键盘弹出时，`Scaffold` 默认 `resizeToAvoidBottomInset: true`
- 底部调试面板（诊断/输出/算法）被键盘推上去，位置不固定
- 用户要求底部面板只能通过拖拽标题栏调整高度

### 3. 系统键盘缺少编程所需按键
- 没有 `Tab` 键（缩进）
- 没有快捷符号输入（`{ }`、`->`、`&&` 等）
- 没有方向键（移动端靠手势）

---

## 方案设计

### 核心思路
**自己实现虚拟键盘**，完全替代系统键盘：
- 拦截系统键盘弹出（编辑器设为 `readOnly: true`）
- 点击编辑区时立即显示自定义键盘（零延迟）
- 键盘内置快捷符号栏、QWERTY 字母区、功能键
- 保留系统键盘切换入口（用于中文输入）

### 键盘布局
```
┌──────────────────────────────────────────────────────┐
│  { }  ( )  [ ]  " "  ' '  ;  #  ->  &  *  =  ==  != │  ← 快捷符号栏（横向滚动）
├──────────────────────────────────────────────────────┤
│  Q  W  E  R  T  Y  U  I  O  P                        │
│   A  S  D  F  G  H  J  K  L                          │
│  ↑  Z  X  C  V  B  N  M  ⌫                          │  ← Shift + 字母 + Backspace
├──────────────────────────────────────────────────────┤
│  中/英   Tab      ␣       ↵      完成               │  ← 功能键区
└──────────────────────────────────────────────────────┘
```

---

## 实现文件

| 文件 | 说明 |
|------|------|
| `lib/widgets/custom_keyboard.dart` | 自定义键盘组件 |
| `lib/widgets/editor_panel.dart` | 编辑器面板，暴露焦点和输入操作 |
| `lib/screens/ide_screen.dart` | 主屏幕，集成键盘和状态控制 |

---

## 关键技术点

### 1. 拦截系统键盘

`re_editor` 的 `CodeEditor` 支持 `readOnly` 参数。当 `readOnly: true` 时，内部不会打开 `TextInputConnection`，系统键盘不会弹出。

```dart
// editor_panel.dart
bool _readOnly = true;

CodeEditor(
  readOnly: _readOnly,
  showCursorWhenReadOnly: true,  // 只读模式下仍显示光标
  ...
)
```

### 2. 手动请求焦点（解决 readOnly 不响应点击）

`re_editor` 在 `readOnly` 模式下，内部 `_CodeSelectionGestureDetector` 不会自动请求焦点。导致第一次点击编辑区时焦点不变化，键盘不弹出。

**解决方案**：在 `CodeEditor` 外层包 `Listener`，监听 `onPointerDown`，手动调用 `requestFocus()`。

```dart
Listener(
  onPointerDown: (_) => widget.onTap?.call(),
  behavior: HitTestBehavior.translucent,
  child: CodeEditor(...),
)
```

`Listener` 比 `GestureDetector` 更底层，不会被 `re_editor` 内部的手势识别器拦截。

### 3. 手动控制键盘显示（抛弃 FocusNode 监听）

最初尝试监听 `FocusNode.hasFocus` 来控制键盘显示，但遇到严重问题：
- 点击自定义键盘按键时，Flutter 焦点系统会短暂清除编辑器焦点
- 点击工具栏、模板栏等任意区域也会导致焦点丢失
- 键盘频繁闪烁/消失

**解决方案**：完全抛弃 `FocusNode` 监听，使用纯手动状态 `_showKeyboard`。

```dart
// ide_screen.dart
bool _showKeyboard = false;

void _openKeyboard() {
  if (!_showKeyboard) setState(() => _showKeyboard = true);
}

void _closeKeyboard() {
  if (_showKeyboard) setState(() => _showKeyboard = false);
}
```

键盘显示条件：
```dart
final showCustomKeyboard = _showKeyboard && !_isSystemKeyboardActive;
```

### 4. 防止键盘点击抢走焦点

自定义键盘本身也是一个 widget 树，点击键盘按键时默认可能触发焦点转移。

**解决方案**：键盘根节点包 `FocusScope(canRequestFocus: false)`，阻止整个键盘子树请求焦点。

```dart
FocusScope(
  canRequestFocus: false,
  child: CustomKeyboard(...),
)
```

### 5. 解决光标消失问题

`readOnly: true` 模式下，每次通过 `_controller.replaceSelection()` 插入文本后，`re_editor` 内部光标显示状态可能被重置，导致光标视觉上"消失"。

**解决方案**：在每个输入操作方法后，显式重新请求焦点 + 确保光标可见。

```dart
void _keepActive() {
  _focusNode.requestFocus();
  _controller.makeCursorCenterIfInvisible();
}

void insertText(String text) {
  _controller.replaceSelection(text);
  _keepActive();
}

void backspace() {
  _controller.deleteBackward();
  _keepActive();
}
```

### 6. 中/英文输入切换

C 语言编程基本不需要中文，但注释或字符串中可能需要。

**实现**：
- 自定义键盘上放置"中"按钮
- 点击后：`_editorKey.currentState?.setReadOnly(false)` + `TextInput.show()`
- 系统键盘弹出，可正常输入中文
- 右下角悬浮"英"按钮，点击后切回自定义键盘：`TextInput.hide()` + `setReadOnly(true)`

---

## 遇到的问题与解决

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 系统键盘仍然弹出 | `_readOnly` 初始为 `false` | 改为 `_readOnly = true` |
| 初始点击编辑区键盘不弹出 | `re_editor` 内部手势消费了点击事件，外层 `GestureDetector` 不触发 | 改用 `Listener.onPointerDown` |
| 点击任意按键键盘就消失 | Flutter 焦点系统点击外部时清除焦点 | 抛弃 `FocusNode` 监听，改用 `_showKeyboard` 手动状态 |
| 键盘按键点击后编辑器失焦 | 键盘子树可能请求焦点 | 根节点包 `FocusScope(canRequestFocus: false)` |
| 文本无法输入 | 误以为 `readOnly` 会拦截 `replaceSelection` | 源码确认 `replaceSelection` 无 `readOnly` 检查，实际可用 |
| 输入时光标消失 | `readOnly` 模式下每次操作后光标状态被重置 | 每次操作后调用 `_focusNode.requestFocus()` + `makeCursorCenterIfInvisible()` |
| 键盘布局右侧溢出 | 按键固定 32px 宽度，小屏幕放不下 | 改用 `Expanded` 自适应宽度 |
| 中英切换后系统键盘不消失 | `TextInput.hide()` 调用时机问题 | 先 `hide()` 再 `setReadOnly(true)` |

---

## 最终架构

```
IdeScreen (State)
  ├── _showKeyboard: bool          # 手动控制键盘显示
  ├── _isSystemKeyboardActive: bool # 是否使用系统键盘（中文输入）
  │
  ├── EditorPanel
  │     ├── _readOnly: true        # 拦截系统键盘
  │     ├── _focusNode: FocusNode  # 暴露给外部操作
  │     ├── _controller            # 文本控制器
  │     └── Listener > CodeEditor  # Listener 捕获点击
  │
  ├── CustomKeyboard (Stack Positioned bottom)
  │     ├── FocusScope(canRequestFocus: false)
  │     ├── 快捷符号栏 (ListView)
  │     ├── QWERTY 字母区
  │     └── 功能键区 (中/英, Tab, Space, Enter, 完成)
  │
  └── 系统键盘切换悬浮按钮 (英)
```

---

## 使用方式

```dart
// 显示自定义键盘
_editorKey.currentState?.setReadOnly(true);
_showKeyboard = true;

// 显示系统键盘（中文）
_editorKey.currentState?.setReadOnly(false);
SystemChannels.textInput.invokeMethod('TextInput.show');

// 隐藏键盘
_showKeyboard = false;
SystemChannels.textInput.invokeMethod('TextInput.hide');
```

---

## 效果

- ✅ 点击编辑区，自定义键盘**立即**弹出，零延迟
- ✅ 底部面板始终固定，不受键盘影响
- ✅ 支持 `Tab` 缩进、快捷符号、退格、回车
- ✅ 输入时光标正常闪烁，不消失
- ✅ 可切换系统键盘输入中文
- ✅ 键盘布局自适应屏幕宽度，无溢出
