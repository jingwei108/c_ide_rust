# Flutter Overlay 弹窗 Material 缺失导致双黄线/红屏 + 运行无反应修复记录

## 问题背景

- **设备**：OPPO PKT110 (Android, Vulkan/Impeller)
- **触发方式**：点击悬浮球菜单展开任意面板（学习进度、监视变量等）
- **表现**：
  1. 悬浮球弹窗内所有文字下方出现**双黄线**（图一）
  2. `监视变量`面板直接红屏报错 `No Material widget found` + `BOTTOM OVERFLOWED BY 99419 PIXELS`（图二）
  3. 点击绿色运行按钮（▶）**无任何反应**，底部输出框始终显示"等待执行"
  4. 点击单步执行按钮同样无反应

---

## 问题一：双黄线 + 红屏（Material 缺失）

### 根因

`FloatingPanelPopup` 通过 `OverlayEntry` 挂载到 `Overlay` 上。`Overlay` 虽然在 `MaterialApp` 内部，但 `OverlayEntry` 的内容**不会自动继承** `Scaffold` 提供的 `Material` 上下文。

底部 Tab 正常，因为它们位于 `Scaffold → body → Column` 内，`Scaffold` 自动包裹了 `Material`。

当弹窗内部出现需要 `Material` 祖先的组件时：
- `WatchTab` 中的 `TextField` → 直接触发**红屏错误**
- `ProgressTab` 中的 `TextButton`（重置进度）→ 触发 Flutter 调试绘制异常，表现为文字下方的**双黄线**

### 修复

在 `FloatingPanelPopup` 最外层包裹 `Material(type: MaterialType.transparency)`，为弹窗内部所有 Material 组件提供正确的祖先环境：

```dart
// lib/widgets/floating_panel_popup.dart
return Positioned.fill(
  child: Material(
    type: MaterialType.transparency,
    child: GestureDetector(
      onTap: _close,
      child: AnimatedBuilder(...),
    ),
  ),
);
```

同时，关闭按钮的 `InkWell` 也一并替换为 `GestureDetector`（弹窗内无需水波纹，且消除多余的 Material 依赖风险）：

```dart
GestureDetector(
  onTap: _close,
  child: Container(
    decoration: BoxDecoration(borderRadius: BorderRadius.circular(12)),
    padding: const EdgeInsets.all(4),
    child: const Icon(Icons.close, size: 18, color: Colors.grey),
  ),
)
```

### 验证

- 将`学习进度`/`监视变量`移到底部 Tab → 显示正常（无黄线、无红屏）
- 悬浮球弹窗中打开同一面板 → 修复前双黄线/红屏，修复后正常

---

## 问题二：点击运行/单步执行无反应

### 根因

`ide_notifier.dart` 中的 `run()` 和 `step()` 直接调用 Rust 后端：
- `rust.runCode()` → 后端检查 `session.compile.compiled`，若未编译返回 `"程序尚未编译。请先编译代码。"`
- `rust.stepNext()` → 同样检查 `compiled`，未编译返回 `StepStatus::Trap`

**前端工具栏没有独立的"编译"按钮**，用户只能点击运行/单步，导致后端始终返回未编译错误。而 `state.error` 在 UI 上**从未被监听和显示**，用户完全看不到反馈，表现为"点击无反应"。

### 修复

#### 1. 运行/单步前自动编译

```dart
// lib/providers/ide_notifier.dart
Future<void> run() async {
  if (!state.isRunning) {
    await compile();
    if (state.hasErrors) {
      state = state.copyWith(error: '请先修复编译错误');
      return;
    }
  }
  // ... 原有运行逻辑
}

Future<void> step() async {
  if (!state.isRunning) {
    await compile();
    if (state.hasErrors) {
      state = state.copyWith(error: '请先修复编译错误');
      return;
    }
  }
  // ... 原有单步逻辑
}
```

- 若当前未在运行中，先自动编译当前代码
- 编译失败（有错误）时直接返回，不再调用后端
- 已在运行中（如等待输入恢复）则跳过编译，避免重复

#### 2. 错误信息 UI 透出

在 `IdeScreen` 中添加 `ref.listen` 监听 `state.error`：

```dart
// lib/screens/ide_screen.dart
ref.listen(ideProvider, (prev, next) {
  if (next.error != null && next.error != prev?.error) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(next.error!),
            duration: const Duration(seconds: 3),
            behavior: SnackBarBehavior.floating,
          ),
        );
      }
    });
  }
});
```

### 验证

- 打开 App → 默认代码 `Hello, Cide!` → 点击运行按钮 → 自动编译 → 输出 `Hello, Cide!`
- 修改代码引入语法错误 → 点击运行 → 弹出 SnackBar `"请先修复编译错误"`
- 点击单步执行 → 自动编译 → 进入单步调试模式

---

## 修复文件

| 文件 | 修改内容 |
|------|----------|
| `lib/widgets/floating_panel_popup.dart` | 最外层包裹 `Material`；关闭按钮 `InkWell` → `GestureDetector` |
| `lib/providers/ide_notifier.dart` | `run()` / `step()` 增加自动编译逻辑 |
| `lib/screens/ide_screen.dart` | 增加 `state.error` 监听与 `SnackBar` 提示 |

---

## 核心教训

1. **OverlayEntry 不继承 Scaffold 的 Material** — 任何通过 `Overlay` 显示的弹窗/浮层，如果内部包含 `TextField`、`TextButton`、`InkWell` 等 Material 组件，必须自行包裹 `Material`
2. **运行前必须编译** — IDE 中"运行"按钮应自动触发编译（或至少给出明确提示），不能假设用户已手动编译
3. **后端错误必须前端透出** — `state.error` 必须在 UI 中有对应的监听和展示机制（SnackBar / Dialog / Banner），否则用户面对"静默失败"毫无头绪
