# CideFlutter Testing Framework

> [中文版](FLUTTER_TESTING.md)
>
> This document describes the testing architecture, infrastructure, conventions, and execution commands for `CideFlutter`.

## 1. Test Layers

```
CideFlutter/
├── test/                    # Unit tests / Widget tests
│   ├── editor/              # Editor kernel (Document / Controller / Layer)
│   ├── models/              # Data models
│   ├── providers/           # Riverpod Notifiers / Providers
│   ├── services/            # Business services
│   ├── widgets/             # UI components
│   ├── helpers/             # Test helpers and factories
│   └── mocks/               # Mock classes
└── integration_test/        # End-to-end integration tests
    └── app_test.dart
```

| Layer | Goal | Command |
|-------|------|---------|
| Unit tests | Correctness of models, controllers, services, and providers | `flutter test` |
| Widget tests | Component rendering, user interaction, and provider integration | `flutter test test/widgets/...` |
| Integration tests | Real app launch → compile → run → output verification | `flutter test -d windows integration_test/` |

## 2. Core Infrastructure

### 2.1 Provider Injection and Mocking

All Rust calls are abstracted through `RustApiService` and replaced with `MockRustApiService` in tests:

```dart
final container = ProviderContainer(
  overrides: [
    rustApiServiceProvider.overrideWithValue(mock),
  ],
);
```

Common helpers:

- `test/helpers/pump_app.dart`
  - `pumpWidget(tester, child: ...)`: pure component tests without providers.
  - `pumpAppWithProviders(tester, builder: ...)`: automatically creates a `ProviderScope` and listens to `ideProvider` / `unifiedProvider` to prevent auto-dispose.

- `test/mocks/rust_api_service_mock.dart`
  - `MockRustApiService`: a `mocktail`-based mock.
  - Prebuilt factories: `compileSuccess` / `compileFailure` / `runSuccess` / `runFailure` / `dummyStepPayload` / `emptyBatch`, etc.

### 2.2 Rust API Stubs

`test/helpers/rust_api_stubs.dart` provides common stubs:

```dart
stubCompileSuccess(mock);
stubCompileFailure(mock, message: 'missing semicolon');
stubRunSuccess(mock, output: 'hello\n');
stubUnifiedPipeline(mock); // returns a StreamController for pushing batches
stubEmptyVisualization(mock);
```

### 2.3 State Factories

`test/helpers/ide_state_factory.dart` provides:

- `idleIdeState()` / `ideStateWithDiagnostic()` / `ideStateWithOutput()`
- `playbackState(cache)` / `playbackStateWithMemory(...)` / `collectingState()` / `pausedState(cache)`
- `stepPayload(...)` / `variable(...)` / `frame(...)` / `algorithmMatch(...)`
- `arraySnapshot(...)` / `pointerSnapshot(...)` / `memoryRegion(...)` / `memoryFragment(...)` / `heapStats(...)` / `visEvent(...)`

These factories reduce boilerplate when constructing FRB-generated objects in tests.

## 3. FRB Type Notes

Generated types differ between the stream path and the UI state path:

| Scenario | Type | Example |
|----------|------|---------|
| Stream batch | `stream.StepStreamBatch` / `StepPayloadRef` / `StepPayloadDelta` | Used by `runAutoStepsStream` |
| UI / Provider state | `types.StepPayload` / `ArraySnapshot` / `PointerSnapshot` | Used in `frameCache` |

Construct test data with the type appropriate to the context.

> `BigInt.zero` cannot be used in `const` contexts; use `BigInt.from(0)` instead.

## 4. Writing Conventions

### 4.1 Must-follow

1. **Any widget that reads providers must be wrapped in a `ProviderScope`**
   - Prefer `pumpAppWithProviders(...)`.
2. **Pay attention to viewport size in widget tests**
   - Widgets such as `ExecutionControlPanel` and `ArrayVisualizer` may overflow in small viewports. Use `SizedBox`, `ClipRect`, or adjust `tester.view.physicalSize`.
3. **Use `pump()` instead of `pumpAndSettle()` for continuous animations**
   - The pulse animation in `ArrayVisualizer` and the breathing animation in `FloatingOrbWidget` never settle; `pumpAndSettle` will time out.
4. **Correctness assertions first**
   - Do not only assert "widget exists"; verify values, states, and mapping relationships.
   - Example: `AnimatedOpacity.opacity == 0.35` for NULL pointers; arrays with more than 40 elements render only the first 40.

### 4.2 Recommended Pattern

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

## 5. Running Tests

```bash
# All unit/widget tests
cd CideFlutter && flutter test

# A single directory
cd CideFlutter && flutter test test/editor/
cd CideFlutter && flutter test test/widgets/
cd CideFlutter && flutter test test/providers/

# A single file
cd CideFlutter && flutter test test/editor/cide_document_test.dart

# Integration tests (desktop)
cd CideFlutter && flutter test -d windows integration_test/
```

## 6. Current Coverage Overview

| Module | Main test files | Correctness assertions |
|--------|----------------|------------------------|
| Editor kernel | `editor/cide_document_test.dart` | `undo`/`redo` text recovery; `lineStartOffset`/`lineEndOffset` boundaries |
| Find/Replace | `editor/find_replace_controller_test.dart` | Regex/plain matching, cycling, invalid regex handling |
| Autocomplete | `editor/autocomplete_controller_test.dart` | Prefix filtering, cycling selection |
| Syntax highlight | `editor/syntax_highlight_layer_test.dart` | Empty/keyword lines render without crashing; cache clear |
| Compile/Run/Unified | `providers/compile_run_unified_test.dart` | Compile success/failure, stream finished/trapped, seek, stepNext |
| Unified correctness | `providers/unified_notifier_correctness_test.dart` | Differential decoding, frame-cache window sync, memory state merge, heatmap, currentVariables |
| Array visualization | `widgets/array_visualizer_test.dart` | 40-element truncation, highlighted/swapped indices |
| Array tab | `widgets/array_vis_tab_test.dart` | `visEvents` and `semanticLabel` highlight parsing |
| Memory map | `widgets/memory_map_visualizer_test.dart` | Heap stats values, block tap BottomSheet details |
| Pointer visualization | `widgets/pointer_arrow_widget_test.dart` | Four status labels, NULL opacity = 0.35 |
| Variable history | `widgets/var_history_tab_test.dart` | Change count, current value |
| Integration | `integration_test/app_test.dart` | Real app launch, run, and step output verification |

## 7. Test Metrics

- Unit/Widget tests: `221 passed` (as of 2026-06-16)
- Integration tests: `6 passed` (Windows desktop)

---

> Core principle: **Tests are not for boasting pass rates, but for honestly discovering problems.** Any failure should be recorded and fixed; modifying test expectations to beautify data is prohibited.
