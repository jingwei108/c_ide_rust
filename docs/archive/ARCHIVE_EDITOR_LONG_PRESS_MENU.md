# 编辑器长按上下文菜单实现记录

> 为 Cide Flutter 编辑器实现移动端长按手势菜单：复制 / 粘贴 / 全选 / 词典，附带智能选词与原生选取器手柄。

---

## 问题背景

### 1. 默认长按无菜单
`re_editor` 在移动端虽然支持长按选词和拖拽选取器手柄，但**没有提供上下文菜单**。用户长按后只能看到选区，无法进行复制、粘贴等操作。

### 2. 系统级长按与编辑器手势冲突
- 如果在 `CodeEditor` 外层包 `GestureDetector` 拦截 `onLongPress`，会和 `re_editor` 内部的 `_CodeSelectionGestureDetector` **竞争手势**，导致原生选词和选取器手柄消失。
- 需要一种方式：**既能弹出自定义菜单，又不破坏编辑器内部的长按选词逻辑**。

### 3. 菜单必须满足移动端体验
- 不抢夺焦点（否则编辑器会失去焦点，`re_editor` 会 `cancelSelection` 并隐藏 handles）
- 菜单样式要符合 IDE 暗色/亮色主题
- 位置要智能：不遮挡选区、超出屏幕自动折叠

---

## 方案设计

### 核心思路
1. **手势层**：使用 `Listener` + `Timer` 检测长按，不与内部 `GestureDetector` 竞争
2. **选词层**：借用 `re_editor` 内部 `_CodeFieldRender.selectWord()` 实现智能选词，空白处手动扩展 1 字符
3. **菜单层**：使用 `OverlayEntry` 自定义横向菜单条，不 push Route、不抢焦点
4. **定位层**：通过 `calculateTextPositionScreenOffset` 获取选区屏幕坐标，动态计算菜单位置

### 交互时序
```
手指按下
  └─→ 启动 600ms Timer
        ├─→ 手指快速抬起（短按）
        │     └─→ Timer 取消，打开键盘
        └─→ 按住 ≥600ms（长按）
              ├─→ Timer 触发
              │     ├─→ 调用 selectWord 选中单词
              │     ├─→ 空白处则扩展选中 1 字符
              │     └─→ OverlayEntry 插入横向菜单
              └─→ 手指抬起
                    └─→ 不打开键盘，保留选区与 handles
```

---

## 实现文件

| 文件 | 说明 |
|------|------|
| `lib/widgets/editor_panel.dart` | 核心实现：长按检测、选词、菜单 Overlay、UI 组件 |

---

## 关键技术点

### 1. 长按与短按区分

`Listener` 的 `onPointerDown` 只启动 Timer，**不立即打开键盘**。在 `onPointerUp` 中根据 Timer 是否已被触发来判断：

```dart
onPointerDown: (event) => _startLongPress(event.position),
onPointerUp: (_) {
  if (_longPressTimer != null) {
    widget.onTap?.call(); // 短按：打开键盘
  }
  _cancelLongPress();
},
```

Timer 触发后必须显式置空，否则 `onPointerUp` 会误判：

```dart
_longPressTimer = Timer(_longPressDuration, () {
  _longPressTimer = null;  // ← 关键
  _longPressStart = null;
  _showContextMenu(position);
});
```

### 2. 通过 dynamic 调用 re_editor 内部 API

`re_editor` 没有暴露坐标转文本位置的公开 API。`CodeEditor` 的 `build` 最外层是 `Stack`，`findRenderObject()` 拿到的是 `RenderStack`，不是 `_CodeFieldRender`。

**解决方案**：通过 `CodeEditor` 内部私有的 `_editorKey`（绑定到 `_CodeField` 的 `GlobalKey`）获取真正的 `RenderObject`：

```dart
final codeEditorState = _codeEditorKey.currentState;
final internalKey = (codeEditorState as dynamic)._editorKey as GlobalKey?;
final renderBox = internalKey?.currentContext?.findRenderObject() as RenderBox?;
```

然后 `dynamic` 调用内部方法：

```dart
// 内部会做 globalToLocal，直接传全局坐标
final range = (renderBox as dynamic).selectWord(position: globalPosition) as CodeLineRange?;
final sel = (renderBox as dynamic).setPositionAt(position: globalPosition) as CodeLineSelection?;
```

### 3. OverlayEntry 菜单不抢焦点

`showMenu` 会 push 一个 `PopupRoute`，抢夺焦点，导致 `re_editor` 触发 `cancelSelection` 并隐藏 handles。

**改用 `OverlayEntry`**：直接插入到 `Navigator` 的 `Overlay` 中，不创建新 Route，编辑器焦点不受影响，选取器手柄保持显示。

### 4. 菜单位置基于选区坐标

通过 `calculateTextPositionScreenOffset` 获取选区左上角和右下角的全局坐标：

```dart
final start = (renderBox as dynamic).calculateTextPositionScreenOffset(
  CodeLinePosition(index: sel.startIndex, offset: sel.startOffset),
  false, // false = 左上角
) as Offset?;

final end = (renderBox as dynamic).calculateTextPositionScreenOffset(
  CodeLinePosition(index: sel.endIndex, offset: sel.endOffset),
  true,  // true = 右下角（含行高）
) as Offset?;
```

位置策略：
- **水平**：以选区中心为锚点居中
- **垂直**：优先放在选区上方；若顶部空间不足（`< safeTop + 8`），则放到选区下方
- **边界保护**：`left` 限制在 `[16, screenWidth - menuWidth - 16]`

### 5. 超出屏幕自动折叠两行

预估菜单宽度（每项约 64px + 分割线）：

```dart
final estimatedWidth = itemCount * 64 + (itemCount - 1) * 1 + 32;
final needsWrap = estimatedWidth > screenWidth - 32;
```

当 `needsWrap == true` 时，`_ContextMenuBar` 将菜单项均分为上下两行：

```dart
final half = (allItems.length / 2).ceil();
Column(
  children: [
    Row(...前 half 项...),
    Divider(height: 1),
    Row(...后 half 项...),
  ],
)
```

---

## 踩坑记录

### 坑 1：菜单没有出现（NoSuchMethodError）
**现象**：长按后选词正常，但菜单没弹出。
**原因**：`_codeEditorKey.currentContext?.findRenderObject()` 拿到的是 `RenderStack`（`CodeEditor.build` 最外层），不是 `_CodeFieldRender`。`dynamic` 调用 `selectWord` 时方法不存在，抛出异常导致 `_showContextMenu` 中断。
**修复**：通过内部 `_editorKey` 获取 `_CodeFieldRender`。

### 坑 2：菜单没有出现（坐标转换两次）
**现象**：修复坑 1 后菜单仍不出现。
**原因**：`_selectWordAt` 中手动做了 `renderBox.globalToLocal(globalPosition)`，但 `selectWord` / `setPositionAt` **内部自己会再做一次 `globalToLocal`**，导致坐标被转换了两次。
**修复**：直接传 `globalPosition`，不再手动转换。

### 坑 3：长按同时触发单击、键盘弹出
**现象**：长按时键盘也弹出来了。
**原因**：`onPointerDown` 中直接调用了 `widget.onTap?.call()`（打开键盘），任何按下都会触发。
**修复**：将 `widget.onTap?.call()` 移到 `onPointerUp` 中，仅在短按时调用。

### 坑 4：选取器手柄消失
**现象**：菜单弹出后，选区变成光标，handles 不见了。
**原因**：最初使用 `showMenu`，它会 push `PopupRoute` 抢夺焦点。`re_editor` 在失去焦点时会 `cancelSelection` + `hideHandle`。
**修复**：改用 `OverlayEntry`，不抢焦点。

---

## 最终效果

### 正常选词
- 长按单词 → `re_editor` 原生选中该词 + 显示拖拽 handles
- 600ms 后灰色横向菜单出现在选区上方
- 菜单项：复制（有选区时显示）/ 粘贴 / 全选 / 词典

### 空白处长按
- `selectWord` 返回 `null`
- 手动调用 `setPositionAt` 定位光标，并扩展选中 **1 个字符/空格**
- 菜单同样弹出，复制项可用（因为已有 1 字符选区）

### 小屏幕/横屏
- 预估宽度超出屏幕时，菜单自动折叠为上下两行，中间以横线分割

---

## 后续可拓展

- **词典功能**：`onDictionary` 回调已预留，可接入本地词库或在线翻译 API
- **更多菜单项**：如「剪切」「注释」「跳转定义」等，直接在 `_ContextMenuBar` 的 `buildItems()` 中追加即可
