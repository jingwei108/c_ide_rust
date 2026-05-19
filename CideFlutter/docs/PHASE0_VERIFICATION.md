# Phase 0 POC 验证报告

> 起始 commit: `8f403c2`  
> 完成 commit: `TBD`  
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
4. ✅ P0 ~ P4 已修复。

---

## Phase 1 目标预览

- 将 `CideEditor` 接入实际 `IdeScreen`，替换 `re_editor` 的编辑区域。
- `IdeScreen._editorKey` 类型改为 `GlobalKey<CideEditorState>()`。
- 接入 Rust 编译/诊断结果到 `RuntimeLayer` / `DiagnosticLayer`。
- 移除 POC 临时代码和调试日志。
