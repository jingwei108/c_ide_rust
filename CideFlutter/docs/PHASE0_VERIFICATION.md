# Phase 0 POC 验证报告

> 起始 commit: `8f403c2`  
> 完成 commit: `5661209`  
> 验证目标：确认 Gesture Proxy 架构的核心假设成立，进入 Phase 1 集成。

---

## 验证结果总览

| 验证项 | 桌面端 | 移动端 | 备注 |
|--------|--------|--------|------|
| V1. 视觉透明度 | ✅ 通过 | ✅ 通过 | EditableText 完全透明，CustomPaint 独占渲染 |
| V2. IME Composing | ✅ 通过 | ✅ 通过 | 中文输入法候选词下划线正确显示，composing 不中断 |
| V3. Pointer 坐标 | ✅ 通过 | ✅ 通过 | 点击坐标缓存正确，hit-test 映射到文本行/列 |
| V4. 滚动同步 | ✅ 通过 | ✅ 通过 | 文本、选区、光标随滚动无漂移 |
| V5. 性能 (FPS) | ✅ 通过 | ✅ 通过 | 桌面端 ~160 FPS，移动端稳定 60 FPS，500 行滚动流畅 |
| V6. 文本同步一致性 | ✅ 通过 | ✅ 通过 | Proxy ↔ Document 双向同步无循环、无丢字符 |

---

## 关键 Bug 修复记录

### P0 — `CustomPaint(size: Size.infinite)` 消费所有点击事件

- **现象**：`Listener` 和 `EditableText` 完全收不到点击事件，导致无法获取焦点、无法移动光标、无法输入。
- **根因**：`CustomPaint` 的 `size: Size.infinite` 导致 `RenderBox.hitTest` 的 bounds 为无限，消费了所有指针事件。
- **修复**：`CustomPaint` 外层包裹 `IgnorePointer`，事件穿透到下层 `EditableText`。
- **文件**：`lib/editor/cide_editor.dart`

### P1 — `shouldRepaint` 比较同一对象的当前值

- **现象**：插入片段/编辑文本后 `CustomPaint` 不重绘，必须滚动后才刷新。
- **根因**：`CideEditorPainter.shouldRepaint` 中 `old.document.text != document.text` 比较的是同一个 `CideDocument` 实例的当前值，永远相等。
- **修复**：在 `CideEditorPainter` 构造函数中快照 `text` / `selection` / `composing`，`shouldRepaint` 比较快照值。
- **文件**：`lib/editor/editor_painter.dart`

### P2 — `_onDocumentChanged` 滥用 `setEditingState` 干扰 IME

- **现象**：系统键盘只能输入几个字符就停止，焦点跳回左上角，中文 composing 中断。
- **根因**：每次 document 变化都调用 `_inputConnection?.setEditingState()` 回传 IME，形成反馈循环；`applyEdit`/`updateSelection`/`updateComposing` 分别触发三次 `_onDocumentChanged`，中间状态错误覆盖 IME 状态。
- **修复**：
  1. 移除 `_onDocumentChanged` 中的 `setEditingState`（IME 发起的编辑不需要回传）。
  2. `CideDocument` 新增 `applyEditSync` / `setTextSync` 批量更新 API，只触发一次 `notifyListeners`。
- **文件**：`lib/editor/cide_editor.dart`, `lib/editor/cide_document.dart`

### P3 — `IdeScreen` 自动键盘切换误判

- **现象**：过几秒自绘键盘抢夺焦点，IME 连接被强制断开。
- **根因**：`IdeScreen.didChangeDependencies` 中根据 `viewInsetsBottom` 自动切回自定义键盘，IME 候选词面板弹出/收起时 `viewInsets` 波动导致误判。
- **修复**：移除自动切换逻辑，键盘切换只由用户明确操作触发。
- **文件**：`lib/screens/ide_screen.dart`

### P4 — Android `MainActivity` 包名与 `applicationId` 不匹配

- **现象**：移动端闪退 `ClassNotFoundException: com.cide.app.MainActivity`。
- **根因**：`applicationId` 已改为 `com.cide.app`，但 `MainActivity.kt` 仍在 `com.example.cide` 包中，`AndroidManifest.xml` 使用 `.MainActivity` 相对路径解析失败。
- **修复**：`AndroidManifest.xml` 中 `android:name=".MainActivity"` → `android:name="com.example.cide.MainActivity"`。
- **文件**：`android/app/src/main/AndroidManifest.xml`

### P5 — Gutter 取 `scrollController.offset` 时 ScrollController 尚未 attach

- **现象**：进入 IDE 初始界面直接红屏崩溃，assert `'positions.isNotEmpty': ScrollController not attached to any scroll views`。
- **根因**：`_buildGutter` 在首次 `build` 时直接访问 `_editorKey.currentState?.scrollController.offset`，此时 `SingleChildScrollView` 尚未完成 attach，`hasClients == false`。
- **修复**：访问 `offset` 前增加 `scrollController.hasClients` 保护，未 attach 时回退为 `0.0`。
- **文件**：`lib/widgets/editor_panel_v2.dart`

### P6 — 系统键盘模式切换后 TextInputConnection 未建立

- **现象**：点击「中/英」切换到系统键盘后无法输入；必须重新点击编辑器才能恢复。
- **根因**：`IdeScreen._showSystemKeyboard()` 只调用 `SystemChannels.textInput.invokeMethod('TextInput.show')`，但底层 `CideEditorState` 的 `_isSystemKeyboardActive` 仍为 `false`，`_attachInputConnection()` 永远不会执行；`TextInputConnection` 未建立。
- **修复**：
  1. `EditorPanelV2State` 暴露 `showSystemKeyboard()` / `showCustomKeyboard()` 公共 API。
  2. `IdeScreen` 在切换键盘时调用 `_editor?.showSystemKeyboard()` / `_editor?.showCustomKeyboard()`，由 `CideEditorState` 内部完成 `_attachInputConnection()` / `_detachInputConnection()`。
- **文件**：`lib/widgets/editor_panel_v2.dart`, `lib/screens/ide_screen.dart`

### P7 — 关闭键盘时系统键盘模式被自动抢夺回自绘键盘

- **现象**：处于系统键盘模式时，点击空白处关闭键盘或上下滑动收起键盘，自动跳回自绘键盘；系统键盘状态无法保持。
- **根因**：`_closeAllKeyboards()` 中 `if (_isSystemKeyboardActive) { _showCustomKeyboard(); }` 把关闭键盘操作变成了切换键盘模式。
- **修复**：
  1. `_closeAllKeyboards()` 只隐藏键盘，不改变 `_isSystemKeyboardActive` 模式。
  2. `_openKeyboard()` 根据当前模式恢复对应键盘（系统键盘模式时重新 `showSystemKeyboard()`）。
  3. 只有显式的「中/英」/「英」按钮才能切换模式。
- **文件**：`lib/screens/ide_screen.dart`

### P8 — 中文输入时 composing 中间态被 `state.source` 回写覆盖，导致乱码累积

- **现象**：使用系统键盘输入中文，候选词上屏后拼音残留（如 `chang'wen`）与最终汉字混合，出现 `chang'wchang'wenchang...` 式乱码；光标随机跳回文档开头。
- **根因**：`EditorPanelV2._onDocumentChanged` 使用 `addPostFrameCallback` 延迟更新 `ideProvider.state.source`；在此期间 IME 继续输入，`_document.text` 已更新为新值，但 provider 仍持有旧值；下一帧 `build` 中 `_document.setText(state.source)` 把 document **回滚**到旧值，proxy 与 IME 状态错乱，diff 计算崩坏。
- **修复**：`EditorPanelV2State` 新增 `_documentDirty` 标志；本地编辑期间（IME / 自绘键盘输入）`build` 中禁止把滞后的 `state.source` 回写到 `_document`；post frame callback 重置标志后才允许外部 source 同步。
- **文件**：`lib/widgets/editor_panel_v2.dart`

### P9 — 行尾输入时光标在第四行末尾与第五行开头之间随机跳动

- **现象**：在 `printf("Hello, Cide!\n");` 行尾输入中文或英文时，`SelectionLayer` 绘制的光标随机出现在当前行末尾或下一行开头（如第四行末 ↔ 第五行首来回跳动）。
- **根因**：`CideDocument._apply` 使用 `_rebuildLineOffsetsFromOffset` 增量重建行首索引。该算法先用**旧的** `_lineStartOffsets` 计算 `startLine`，当新插入文本使当前行变长后，`offsetToLine` 把 `startOffset` 误判到下一行；截断点错误导致重建出来的 `_lineStartOffsets` 顺序被破坏（出现后面的值比前面还小）。后续 `offsetToLine` 的 binary search 在乱序数组上返回随机行号，`DocPosition` 行号错误，光标位置随之乱跳。
- **修复**：
  1. `_apply` 中放弃增量重建，改为调用 `_rebuildLineOffsets()` 全量重建。对于通常几十到几百行的 C 代码，O(n) 扫描性能完全可接受。
  2. `offsetToPosition` 增加边界处理：当 `offset` 正好等于某行行首时，归属到**上一行末尾**，避免光标在行尾被误判到下一行开头。
  3. 系统键盘模式下 `_onDocumentChanged` 不再回传 `selection` 给 proxy，防止 V2 和 IME 来回争夺光标位置。
- **文件**：`lib/editor/cide_document.dart`, `lib/editor/cide_editor.dart`

### P10 — Windows 桌面端系统键盘状态下不显示「英」切换按钮

- **现象**：在 Windows 桌面端切换到系统键盘后，屏幕右下角没有出现「英」悬浮按钮，无法切回自绘键盘。
- **根因**：显示「英」按钮的条件包含 `isSystemKeyboardReallyVisible = viewInsetsBottom > 50`。Windows 桌面端使用物理键盘时 `MediaQuery.viewInsets` 不会变化，始终为 `false`，导致按钮被隐藏。
- **修复**：桌面端（`Platform.isWindows || Platform.isMacOS || Platform.isLinux`）只要 `_isSystemKeyboardActive == true` 就直接显示切换按钮，不依赖 `viewInsets`；移动端仍保留 `viewInsets` 判断，避免系统键盘收起后按钮悬浮遮挡内容。
- **文件**：`lib/screens/ide_screen.dart`

---

## 架构决策确认

| 决策 | 结论 |
|------|------|
| Gesture Proxy 模式可行 | ✅ 确认 — `EditableText` 透明代理 + `CustomPaint` 独占渲染方案成立 |
| 双向同步策略 | ✅ 确认 — `_syncing` 锁 + 批量更新有效防止循环 |
| 输入分流 | ✅ 确认 — 系统 IME → `TextInputConnection` → `updateEditingValue` → document；自定义键盘 → `CideDocument` API → `_syncToProxy()` |
| 滚动架构 | ✅ 确认 — 外层 `SingleChildScrollView` + `_scrollController.hasClients` 安全访问 |
| 无物理快捷键 | ✅ 确认 — Phase 0 明确不支持 Ctrl+A/C/V/Z 等快捷键，由 `EditableText` 原生处理或 Phase 1 再议 |

---

## 进入 Phase 1 标准（已全部满足）

1. ✅ V1 ~ V4 全部通过（核心架构假设成立）。
2. ✅ V5 通过，500 行滚动流畅，桌面端 ~160 FPS / 移动端 60 FPS。
3. ✅ V6 通过，同步逻辑正确，无丢字符。
4. ✅ P0 ~ P9 已修复。

---

## Phase 1 目标预览

- 将 `CideEditor` 接入实际 `IdeScreen`，替换 `re_editor` 的编辑区域。
- `IdeScreen._editorKey` 类型改为 `GlobalKey<CideEditorState>()`。
- 接入 Rust 编译/诊断结果到 `RuntimeLayer` / `DiagnosticLayer`。
- 移除 POC 临时代码和调试日志。
