import 'dart:async';
import 'package:cide/models/ide_state.dart';
import 'package:cide/models/learning_progress.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/services/rust_api_service.dart';
import 'package:cide/src/rust/session.dart';
import 'package:cide/src/rust/unified/stream.dart' as stream;
import 'package:cide/src/rust/unified/types.dart' as types;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../mocks/rust_api_service_mock.dart';

ProviderContainer _createContainer(MockRustApiService mock) {
  final container = ProviderContainer(
    overrides: [rustApiServiceProvider.overrideWithValue(mock)],
  );
  // AutoDisposeNotifier 在没有 listener 时会被 dispose；保持存活以便读取 state。
  container.listen(ideProvider, (prev, next) {});
  container.listen(unifiedProvider, (prev, next) {});
  return container;
}

void main() {
  setUpAll(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('CompileNotifier compileOnly', () {
    test('sets success state and diagnostics on successful compile', () async {
      final mock = MockRustApiService();
      when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
        (_) async => compileSuccess(),
      );
      when(
        () => mock.inferIntentFromSource(source: any(named: 'source')),
      ).thenAnswer((_) async => const []);

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await Future.delayed(Duration.zero); // _loadProgress
      final success = await notifier.compileOnly();

      expect(success, isTrue);
      final state = container.read(ideProvider);
      expect(state.isCompiling, isFalse);
      expect(state.output, '编译成功');
      expect(state.diagnostics, isEmpty);
      expect(state.learningProgress.totalCompiles, 1);
      expect(state.learningProgress.successfulCompiles, 1);
    });

    test('sets failure state and diagnostics on compile error', () async {
      final mock = MockRustApiService();
      when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
        (_) async => compileFailure(message: '缺少分号'),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await Future.delayed(Duration.zero);
      final success = await notifier.compileOnly();

      expect(success, isFalse);
      final state = container.read(ideProvider);
      expect(state.isCompiling, isFalse);
      expect(state.output, '编译失败');
      expect(state.diagnostics.length, 1);
      expect(state.diagnostics.first.message, '缺少分号');
      expect(state.learningProgress.failedCompiles, 1);
    });
  });

  group('CompileNotifier compile', () {
    test('launches unified mode after successful compile', () async {
      final mock = MockRustApiService();
      final batchController = StreamController<stream.StepStreamBatch>();

      when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
        (_) async => compileSuccess(),
      );
      when(
        () => mock.inferIntentFromSource(source: any(named: 'source')),
      ).thenAnswer((_) async => const []);
      when(
        () => mock.compileAndRunMulti(files: any(named: 'files')),
      ).thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(
        () => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')),
      ).thenAnswer((_) => batchController.stream);
      when(() => mock.getMemoryRegions()).thenAnswer((_) async => const []);
      when(() => mock.getMemorySize()).thenAnswer((_) async => 1024 * 1024);
      when(() => mock.getMemoryFragments()).thenAnswer((_) async => const []);
      when(() => mock.getHeapStats()).thenAnswer(
        (_) async => const HeapStats(
          totalHeap: 0,
          allocated: 0,
          fragmented: 0,
          fragmentationRate: 0,
        ),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await Future.delayed(Duration.zero);
      await notifier.compile();

      // 编译成功后统一模式进入 collecting 阶段。
      final unifiedState = container.read(unifiedProvider);
      expect(unifiedState.phase, ExecutionPhase.collecting);
      expect(unifiedState.isPlaying, isTrue);

      await batchController.close();
    });
  });

  group('RunNotifier run', () {
    test('runs code and updates output when compiled', () async {
      final mock = MockRustApiService();
      when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
        (_) async => compileSuccess(),
      );
      when(
        () => mock.inferIntentFromSource(source: any(named: 'source')),
      ).thenAnswer((_) async => const []);
      when(() => mock.runCode()).thenAnswer(
        (_) async => runSuccess(output: 'hello\n'),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await Future.delayed(Duration.zero);
      await notifier.run();

      final state = container.read(ideProvider);
      expect(state.isRunning, isFalse);
      expect(state.output, 'hello\n');
      expect(state.error, isNull);
    });

    test('stops and reports error when run fails', () async {
      final mock = MockRustApiService();
      when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
        (_) async => compileSuccess(),
      );
      when(
        () => mock.inferIntentFromSource(source: any(named: 'source')),
      ).thenAnswer((_) async => const []);
      when(() => mock.runCode()).thenAnswer(
        (_) async => runFailure(error: '除以零'),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await Future.delayed(Duration.zero);
      await notifier.run();

      final state = container.read(ideProvider);
      expect(state.isRunning, isFalse);
      expect(state.error, '除以零');
    });
  });

  group('RunNotifier step', () {
    test('steps to finished and updates current line', () async {
      final mock = MockRustApiService();
      when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
        (_) async => compileSuccess(),
      );
      when(
        () => mock.inferIntentFromSource(source: any(named: 'source')),
      ).thenAnswer((_) async => const []);
      when(() => mock.stepNext()).thenAnswer(
        (_) async => stepResult(
          status: StepStatus.finished,
          currentLine: 5,
          output: 'done',
        ),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(ideProvider.notifier);
      await Future.delayed(Duration.zero);
      await notifier.step();

      final state = container.read(ideProvider);
      expect(state.isRunning, isFalse);
      expect(state.isStepMode, isFalse);
      expect(state.currentLine, 5);
      expect(state.output, 'done');
    });
  });

  group('UnifiedNotifier compileAndRun', () {
    test('enters error phase when compileAndRunMulti fails', () async {
      final mock = MockRustApiService();
      when(
        () => mock.compileAndRunMulti(files: any(named: 'files')),
      ).thenAnswer((_) async => unifiedRunFailure(error: '编译错误'));

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      final state = container.read(unifiedProvider);
      expect(state.phase, ExecutionPhase.error);
      expect(state.errorMessage, '编译错误');
    });
  });

  group('UnifiedNotifier stream batch handling', () {
    test('collects frames from stream and finishes', () async {
      final mock = MockRustApiService();
      final batchController = StreamController<stream.StepStreamBatch>();

      when(
        () => mock.compileAndRunMulti(files: any(named: 'files')),
      ).thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(
        () => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')),
      ).thenAnswer((_) => batchController.stream);
      when(() => mock.getMemoryRegions()).thenAnswer((_) async => const []);
      when(() => mock.getMemorySize()).thenAnswer((_) async => 1024 * 1024);
      when(() => mock.getMemoryFragments()).thenAnswer((_) async => const []);
      when(() => mock.getHeapStats()).thenAnswer(
        (_) async => const HeapStats(
          totalHeap: 0,
          allocated: 0,
          fragmented: 0,
          fragmentationRate: 0,
        ),
      );
      when(() => mock.getHeatmap()).thenAnswer(
        (_) async => types.HeatmapData(lineCounts: const [], maxCount: BigInt.zero),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');
      expect(container.read(unifiedProvider).phase, ExecutionPhase.collecting);

      batchController.add(emptyBatch(finished: true));
      await Future.delayed(const Duration(milliseconds: 50));

      final state = container.read(unifiedProvider);
      expect(state.phase, ExecutionPhase.playback);
      expect(state.isPlaying, isFalse);

      await batchController.close();
    });

    test('enters error phase when stream reports trap', () async {
      final mock = MockRustApiService();
      final batchController = StreamController<stream.StepStreamBatch>();

      when(
        () => mock.compileAndRunMulti(files: any(named: 'files')),
      ).thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(
        () => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')),
      ).thenAnswer((_) => batchController.stream);
      when(() => mock.getMemoryRegions()).thenAnswer((_) async => const []);
      when(() => mock.getMemorySize()).thenAnswer((_) async => 1024 * 1024);
      when(() => mock.getMemoryFragments()).thenAnswer((_) async => const []);
      when(() => mock.getHeapStats()).thenAnswer(
        (_) async => const HeapStats(
          totalHeap: 0,
          allocated: 0,
          fragmented: 0,
          fragmentationRate: 0,
        ),
      );
      when(() => mock.getHeatmap()).thenAnswer(
        (_) async => types.HeatmapData(lineCounts: const [], maxCount: BigInt.zero),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      batchController.add(emptyBatch(trapped: true, trapMessage: '数组越界'));
      await Future.delayed(const Duration(milliseconds: 50));

      final state = container.read(unifiedProvider);
      expect(state.phase, ExecutionPhase.error);
      expect(state.trapMessage, '数组越界');

      await batchController.close();
    });
  });

  group('UnifiedNotifier seek and step', () {
    test('seekToStep within cache updates current step immediately', () async {
      final container = _createContainer(MockRustApiService());
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      final cache = [
        dummyStepPayload(stepIndex: 0, codeLine: 1),
        dummyStepPayload(stepIndex: 1, codeLine: 2),
        dummyStepPayload(stepIndex: 2, codeLine: 3),
      ];
      notifier.state = container.read(unifiedProvider).copyWith(
        phase: ExecutionPhase.playback,
        frameCache: cache,
        maxCollectedStep: 2,
      );

      await notifier.seekTo(1);

      final state = container.read(unifiedProvider);
      expect(state.currentStep, 1);
      expect(state.currentLine, 2);
      expect(state.phase, ExecutionPhase.playback);
    });

    test('stepNextUnified appends payload to cache', () async {
      final mock = MockRustApiService();
      when(() => mock.stepNextUnified()).thenAnswer(
        (_) async => dummyStepPayload(stepIndex: 0, codeLine: 10),
      );
      when(() => mock.getMemoryRegions()).thenAnswer((_) async => const []);
      when(() => mock.getMemorySize()).thenAnswer((_) async => 1024 * 1024);
      when(() => mock.getMemoryFragments()).thenAnswer((_) async => const []);
      when(() => mock.getHeapStats()).thenAnswer(
        (_) async => const HeapStats(
          totalHeap: 0,
          allocated: 0,
          fragmented: 0,
          fragmentationRate: 0,
        ),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      notifier.state = container.read(unifiedProvider).copyWith(
        phase: ExecutionPhase.paused,
      );

      await notifier.stepNext();

      final state = container.read(unifiedProvider);
      expect(state.frameCache.length, 1);
      expect(state.currentStep, 0);
      expect(state.currentLine, 10);
      expect(state.phase, ExecutionPhase.stepMode);
    });
  });
}
