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

---

## 2026-05-15 键盘重设计

将原有字母/数字混合键盘升级为**三种独立模式**，布局与交互重新设计。

### 三种模式

| 模式 | 入口 | 布局 |
|------|------|------|
| **字母模式（ABC）** | 默认进入 | 顶部精简符号栏 + QWERTY 三行 + 底行功能键 |
| **数字模式（123）** | 底行 `123` 键 | 左侧符号栏 `%、/、-、+` + 中间九宫格 `1-9` + 右侧功能键 |
| **符号模式（符）** | 底行 `符` 键 | 左侧分类菜单 + 右侧 4 列可滑动符号网格 |

### 字母模式细节

- **精简符号栏**：横向滚动，收录 `{} () [] "" '' ; # -> & * = == != < > + - / % && || ! \| ^ ~ , . _ :` 共 29 个高频符号，点击直接输入或成对插入
- **QWERTY 区**：三行标准布局，第 2、3 行左右缩进与参考图一致
- **底行**：`123` \| `中` \| `符` \| `Space`（支持滑动移光标）\| `↵` \| `Tab` \| `完成`
- **按键尺寸**：高度 48，圆角 8，水平间隙 3，垂直间隙 6

### 数字模式细节

- **左侧栏**：`%、/、-、+` 四个符号键，与九宫格共用同一高度等分
- **九宫格**：`1-9` 三行三列，0 单独放在底行
- **右侧栏**：`⌫`（长按连删）\| `Tab` \| `↵`
- **底行对齐**：`符`(左栏正下方) \| `返回` \| `0`(对准中间列 2/5/8) \| `空格` \| `完成`(右栏正下方)

### 符号模式细节

- **分类菜单**：`常用 / 运算符 / 比较 / 位移 / 其他`，点击切换高亮
- **符号网格**：4 列，可上下滑动，点击直接输入或成对插入
- **返回按钮**：菜单栏底部内置，点击回到字母模式

### 关键 UI 参数

| 参数 | 值 |
|------|-----|
| 字母键高度 | 48 |
| 符号栏高度 | 36 |
| 按键圆角 | 8 |
| 字母行垂直间隙 | 6 |
| 字母键水平间隙 | 3 |
| 数字区总高度 | 230 |
| 数字模式间隙 | 6 |

---

## Release 构建混淆配置

为提升逆向难度，在 Release 构建中启用了三层轻量混淆，**Debug 构建完全不受影响**。

### 1. Rust 符号剥离 (`native/Cargo.toml`)

```toml
[profile.release]
debug = false
strip = true
```

- Release 构建的 so/dll 去除全部符号表和 DWARF 调试信息
- 反编译后看不到函数名、变量名和源码行号
- **⚠️ Release 调试影响**：若 Rust 层发生 native crash，堆栈中只剩裸地址（如 `0x12345678`），无法直接定位函数名。需依赖日志和复现环境排查

### 2. Dart 官方混淆 (`scripts/build_flutter.py`)

```python
if configuration == "Release":
    flutter_args.extend(["--obfuscate", "--split-debug-info=symbols/"])
```

- Release 构建时类名、方法名、字段名混淆为 `a.b.c`
- 调试符号分离到 `symbols/` 目录，不进入安装包
- 崩溃分析：使用 `flutter symbolize` 配合 `symbols/` 文件可还原堆栈

### 3. Android R8 压缩 (`android/app/build.gradle.kts`)

```kotlin
release {
    isMinifyEnabled = true
    isShrinkResources = true
    proguardFiles(...)
}
```

- 仅作用于 release build type，对 debug 无影响
- 混淆压缩 Java/Kotlin 插件层代码，移除无用资源

---

## 2026-05-16 沉浸编辑模式与手势优化

### 问题背景
- 键盘弹出时，顶部工具栏、模板栏、底部面板仍然占用大量屏幕空间，编辑器可编辑区域被严重压缩
- 用户希望在键盘弹出时自动收起上下栏，腾出最大空间给编辑器
- 需要在编辑器空白处点击或滑动来收起键盘，上下栏自动恢复

### 沉浸编辑模式

键盘（自定义键盘或系统键盘）弹出时，通过 `SizeTransition` 动画平滑收起上下栏：

| 栏位 | 动画方向 | 说明 |
|------|----------|------|
| 顶部工具栏 (`Toolbar`) | `axisAlignment: -1`（从上往下收起） | 运行控制、主题切换等 |
| 模板栏 (`TemplateBar`) | `axisAlignment: 1`（从下往上收起） | 代码模板快捷插入 |
| 底部面板 (`BottomPanel`) | `axisAlignment: 1`（从下往上收起） | 输出/诊断/算法等标签页 |

实现要点：
- `_IdeScreenState` 混入 `SingleTickerProviderStateMixin`，使用 `_barsAnimationController` 驱动动画
- 动画目标值在 `build()` 中通过 `_syncBarsAnimation()` 同步：`1.0`=显示，`0.0`=隐藏
- 系统键盘真实可见性通过 `MediaQuery.of(context).viewInsets.bottom > 50` 检测
- 系统键盘被系统收起（如返回键）后，自动同步 `_isSystemKeyboardActive = false`

```dart
// ide_screen.dart
SizeTransition(
  sizeFactor: _barsAnimation,
  axisAlignment: -1, // 或 1
  child: _buildToolbar(...),
)
```

### 编辑器手势交互

在 `EditorPanel` 的 `Listener` 中处理三种手势：

| 手势 | 行为 |
|------|------|
| 点击代码字符处 | 打开键盘 |
| 点击空白处（空行/行尾/尾部空白） | 关闭键盘 |
| 上下滑动（\|dy\| > 100px 且垂直为主） | 关闭键盘 |
| 长按（>600ms） | 弹出上下文菜单（不受单击逻辑影响） |

**空白检测实现**（避免依赖 `re_editor` 内部私有 API）：

不再使用 `_editorKey` → `selectWord` / `setPositionAt` 的方案（行为不稳定，易误判标点/空白）。改为延迟到 `re_editor` 内部更新光标位置后，读取公开 API `CodeLineEditingController.selection` 判断：

```dart
// editor_panel.dart - onPointerUp
WidgetBinding.instance.addPostFrameCallback((_) {
  final sel = _controller.selection;
  final lineText = _controller.codeLines[sel.baseIndex].text;
  final offset = sel.baseOffset;
  final isBlank = lineText.trim().isEmpty ||
      offset >= lineText.length ||
      offset >= lineText.trimRight().length;
  if (isBlank) {
    widget.onBlankTap?.call(); // 关闭键盘
  } else {
    widget.onTap?.call();      // 打开键盘
  }
});
```

**长按与单击共存**：
- `onPointerUp` 中**先保存 `wasShortPress = _longPressTimer != null`，再立即 `_cancelLongPress()`**
- 这样即使后续空白检测耗时较长，600ms 的长按计时器也已经被安全取消，不会误触发长按菜单
- 滑动检测使用独立的 `_swipeStart` 字段，不受 `_checkLongPressMove` 取消长按的影响

### 修复单击变长按的 Bug

**根因**：旧代码中 `_cancelLongPress()` 在 `_isBlankAt()` 之后执行。`_isBlankAt` 内部调用 `setPositionAt` 可能耗时较长或触发 `re_editor` 内部 rebuild，如果在执行期间 600ms 计时器到期，`_showContextMenu` 就会触发，导致用户的一次单击被错误识别为长按。

**修复**：调整 `onPointerUp` 执行顺序，先立即取消计时器，再做任何可能耗时的操作：

```dart
onPointerUp: (event) {
  final wasShortPress = _longPressTimer != null;
  _cancelLongPress(); // ← 立即取消，避免耗时操作导致误触发
  // ... 滑动检测 ...
  // ... 空白检测 ...
}
```

### 最终效果

- ✅ 键盘弹出时上下栏平滑收起，编辑器自动拉伸占满空间
- ✅ 键盘收起后上下栏自动弹出恢复
- ✅ 点击代码处打开键盘，点击空白处关闭键盘
- ✅ 上下滑动关闭键盘
- ✅ 长按不受影响，仍正常弹出上下文菜单
- ✅ 无单击变长按的误触
