# CideFlutter 测试框架文档

> [English Version](FLUTTER_TESTING_EN.md)
>
> 本文档描述 `CideFlutter` 的测试体系结构、基础设施、编写规范与运行方式。

## 1. 测试分层

```
CideFlutter/
├── test/                    # 单元测试 / Widget 测试
│   ├── editor/              # 编辑器内核（Document / Controller / Layer）
│   ├── models/              # 数据模型
│   ├── providers/           # Riverpod Notifier / Provider
│   ├── services/            # 业务服务
│   ├── widgets/             # UI 组件
│   ├── helpers/             # 测试辅助函数与工厂
│   └── mocks/               # Mock 类
└── integration_test/        # 端到端集成测试
    └── app_test.dart
```

| 层级 | 目标 | 运行方式 |
|------|------|----------|
| 单元测试 | Model / Controller / Service / Provider 逻辑正确性 | `flutter test` |
| Widget 测试 | 组件渲染、用户交互、Provider 集成 | `flutter test test/widgets/...` |
| 集成测试 | 真实 App 启动 → 编译 → 运行 → 输出验证 | `flutter test -d windows integration_test/` |

## 2. 核心基础设施

### 2.1 Provider 注入与 Mock

所有 Rust 调用都通过 `RustApiService` 抽象，并在测试中替换为 `MockRustApiService`：

```dart
final container = ProviderContainer(
  overrides: [
    rustApiServiceProvider.overrideWithValue(mock),
  ],
);
```

通用封装见：

- `test/helpers/pump_app.dart`
  - `pumpWidget(tester, child: ...)`：无 Provider 的纯组件测试。
  - `pumpAppWithProviders(tester, builder: ...)`：自动创建 `ProviderScope`、监听 `ideProvider` / `unifiedProvider` 防止被 dispose。

- `test/mocks/rust_api_service_mock.dart`
  - `MockRustApiService`：基于 `mocktail` 的 Mock。
  - 预置工厂：`compileSuccess` / `compileFailure` / `runSuccess` / `runFailure` / `dummyStepPayload` / `emptyBatch` 等。

### 2.2 Rust API Stubs

`test/helpers/rust_api_stubs.dart` 提供常用预置 stub：

```dart
stubCompileSuccess(mock);
stubCompileFailure(mock, message: '缺少分号');
stubRunSuccess(mock, output: 'hello\n');
stubUnifiedPipeline(mock); // 返回 StreamController，便于测试推送 batch
stubEmptyVisualization(mock);
```

### 2.3 状态工厂

`test/helpers/ide_state_factory.dart` 提供：

- `idleIdeState()` / `ideStateWithDiagnostic()` / `ideStateWithOutput()`
- `playbackState(cache)` / `playbackStateWithMemory(...)` / `collectingState()` / `pausedState(cache)`
- `stepPayload(...)` / `variable(...)` / `frame(...)` / `algorithmMatch(...)`
- `arraySnapshot(...)` / `pointerSnapshot(...)` / `memoryRegion(...)` / `memoryFragment(...)` / `heapStats(...)` / `visEvent(...)`

使用工厂函数可以避免在测试中手写大量 FRB 生成的字段。

## 3. FRB 类型注意事项

`flutter_rust_bridge` 生成的类型在 stream 路径与 UI 路径中存在差异：

| 场景 | 类型 | 示例 |
|------|------|------|
| Stream Batch | `stream.StepStreamBatch` / `StepPayloadRef` / `StepPayloadDelta` | 用于 `runAutoStepsStream` |
| UI / Provider State | `types.StepPayload` / `ArraySnapshot` / `PointerSnapshot` | 用于 `frameCache` |

测试中构造数据时必须使用对应上下文的类型。

> `BigInt.zero` 不能用于 `const` 上下文，统一使用 `BigInt.from(0)`。

## 4. 编写规范

### 4.1 必须遵循

1. **任何读取 Provider 的 Widget 必须包在 `ProviderScope` 中**
   - 优先使用 `pumpAppWithProviders(...)`。
2. **Widget 测试注意视口尺寸**
   - `ExecutionControlPanel`、`ArrayVisualizer` 等组件在小视口下会溢出，测试中应使用 `SizedBox`、`ClipRect` 或调整 `tester.view.physicalSize`。
3. **连续动画组件使用 `pump()` 而非 `pumpAndSettle()`**
   - `ArrayVisualizer` 的脉冲动画、`FloatingOrbWidget` 的呼吸动画不会结束，使用 `pumpAndSettle` 会超时。
4. **正确性断言优先**
   - 不要只断言 "组件存在"，要验证数值、状态、映射关系。
   - 例：NULL 指针的 `AnimatedOpacity.opacity == 0.35`；数组超过 40 个元素时确实只渲染前 40 个。

### 4.2 推荐模式

```dart
testWidgets('renders pointer snapshots from unified state', (tester) async {
  final container = await pumpAppWithProviders(
    tester,
    builder: (_) => const PointerVisTab(isDark: false),
  );

  container.read(unifiedProvider.notifier).state = playbackState([
    stepPayload(
      stepIndex: 0,
      pointerSnapshots: [
        pointerSnapshot(name: 'p', tyName: 'int*', targetName: 'x'),
      ],
    ),
  ]);
  await tester.pump();

  expect(find.text('p'), findsOneWidget);
  expect(find.text('x'), findsOneWidget);
});
```

## 5. 运行命令

```bash
# 全部单元/widget 测试
cd CideFlutter && flutter test

# 单独运行某目录
cd CideFlutter && flutter test test/editor/
cd CideFlutter && flutter test test/widgets/
cd CideFlutter && flutter test test/providers/

# 单个文件
cd CideFlutter && flutter test test/editor/cide_document_test.dart

# 集成测试（桌面端）
cd CideFlutter && flutter test -d windows integration_test/
```

## 6. 当前覆盖概览

| 模块 | 主要测试文件 | 正误性断言示例 |
|------|-------------|----------------|
| 编辑器内核 | `editor/cide_document_test.dart` | `undo`/`redo` 后文本精确恢复；`lineStartOffset`/`lineEndOffset` 边界 |
| 查找替换 | `editor/find_replace_controller_test.dart` | 正则/普通匹配、循环、无效正则处理 |
| 自动补全 | `editor/autocomplete_controller_test.dart` | 前缀过滤、循环选择 |
| 语法高亮 | `editor/syntax_highlight_layer_test.dart` | 空行/关键字绘制不崩溃、缓存清空 |
| 编译/运行/统一模式 | `providers/compile_run_unified_test.dart` | 编译成功/失败、stream finished/trapped、seek、stepNext |
| 统一模式数据正确性 | `providers/unified_notifier_correctness_test.dart` | 差分解码、frame cache 窗口同步、内存状态合并、heatmap、currentVariables |
| 数组可视化 | `widgets/array_visualizer_test.dart` | 40 元素截断、高亮/交换索引存在 |
| 数组 Tab | `widgets/array_vis_tab_test.dart` | visEvents 与 semanticLabel 高亮解析 |
| 内存映射 | `widgets/memory_map_visualizer_test.dart` | 堆统计数值、点击块 BottomSheet 详情 |
| 指针可视化 | `widgets/pointer_arrow_widget_test.dart` | 四种状态标签、NULL opacity=0.35 |
| 变量历史 | `widgets/var_history_tab_test.dart` | 变化次数、当前值 |
| 集成测试 | `integration_test/app_test.dart` | 真实 App 启动、运行、单步，验证输出内容 |

## 7. 测试数据

- 单元/Widget 测试：`221 passed`（截至 2026-06-16）
- 集成测试：`6 passed`（Windows 桌面端）

---

> 本测试框架的核心原则：**测试不是为了标榜通过率，而是为了诚实地发现潜在问题**。任何失败都应如实记录并修复，禁止通过修改测试预期来粉饰数据。
