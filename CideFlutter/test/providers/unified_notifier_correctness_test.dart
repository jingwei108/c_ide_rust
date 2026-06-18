import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:cide/models/unified_state.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/services/rust_api_service.dart';
import 'package:cide/src/rust/session.dart';
import 'package:cide/src/rust/unified/stream.dart' as stream;
import 'package:cide/src/rust/unified/types.dart' as types;

import '../mocks/rust_api_service_mock.dart';

ProviderContainer _createContainer(MockRustApiService mock) {
  final container = ProviderContainer(
    overrides: [rustApiServiceProvider.overrideWithValue(mock)],
  );
  container.listen(ideProvider, (prev, next) {});
  container.listen(unifiedProvider, (prev, next) {});
  return container;
}

void _stubEmptyVisualization(MockRustApiService mock) {
  when(() => mock.getMemoryRegions()).thenAnswer((_) async => const <MemoryRegion>[]);
  when(() => mock.getMemorySize()).thenAnswer((_) async => 1024 * 1024);
  when(() => mock.getMemoryFragments()).thenAnswer((_) async => const <MemoryFragment>[]);
  when(() => mock.getHeapStats()).thenAnswer(
    (_) async => const HeapStats(
      totalHeap: 0,
      allocated: 0,
      fragmented: 0,
      fragmentationRate: 0,
    ),
  );
  when(() => mock.getHeatmap()).thenAnswer(
    (_) async => types.HeatmapData(lineCounts: const [], maxCount: BigInt.from(0)),
  );
}

stream.StepStreamBatch _batch({
  required int cacheStartStep,
  required List<stream.StepPayloadRef> basePayloads,
  required List<stream.StepPayloadDelta> deltas,
  bool finished = false,
  bool trapped = false,
}) {
  return stream.StepStreamBatch(
    symbolTable: const ['main', 'x', 'int', 'arr', 'int', 'p', 'int*', 'target', 'compare'],
    basePayloads: basePayloads,
    deltas: deltas,
    finished: finished,
    trapped: trapped,
    waitingInput: false,
    paused: false,
    currentLine: 0,
    trapMessage: trapped ? 'trap' : null,
    cacheStartStep: cacheStartStep,
  );
}

void main() {
  setUpAll(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('UnifiedNotifier batch decoding correctness', () {
    test('decodes base payload fields and symbol table', () async {
      final mock = MockRustApiService();
      final controller = StreamController<stream.StepStreamBatch>();

      when(() => mock.compileAndRunMulti(files: any(named: 'files')))
          .thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(() => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')))
          .thenAnswer((_) => controller.stream);
      _stubEmptyVisualization(mock);

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      controller.add(_batch(
        cacheStartStep: 0,
        basePayloads: [
          stream.StepPayloadRef(
            stepIndex: 0,
            codeLine: 1,
            funcNameIdx: 0,
            semanticLabelIdx: 0,
            localVars: [
              stream.ApiVarSnapshotRef(
                nameIdx: 1,
                addr: 0x1000,
                isLocal: true,
                tyNameIdx: 2,
                value: '10',
              ),
            ],
            callStack: [],
            visEvents: [],
            heatmapLine: 0,
            heatmapCount: BigInt.from(0),
            accessedVars: [],
            arraySnapshots: [
              stream.ArraySnapshotRef(
                nameIdx: 3,
                elementTyIdx: 4,
                elements: ['3', '1'],
              ),
            ],
            pointerSnapshots: [
              stream.PointerSnapshotRef(
                nameIdx: 5,
                addr: 0x2000,
                tyNameIdx: 6,
                targetAddr: 0x3000,
                targetNameIdx: 7,
                status: types.PointerStatus.valid,
              ),
            ],
          ),
        ],
        deltas: const [],
        finished: true,
      ));
      await Future.delayed(const Duration(milliseconds: 50));

      final state = container.read(unifiedProvider);
      expect(state.frameCache.length, 1);
      final payload = state.frameCache.first;
      expect(payload.stepIndex, 0);
      expect(payload.codeLine, 1);
      expect(payload.funcName, 'main');
      expect(payload.localVars.length, 1);
      expect(payload.localVars.first.name, 'x');
      expect(payload.localVars.first.value, '10');
      expect(payload.arraySnapshots.first.name, 'arr');
      expect(payload.arraySnapshots.first.elements, ['3', '1']);
      expect(payload.pointerSnapshots.first.name, 'p');
      expect(payload.pointerSnapshots.first.status, types.PointerStatus.valid);

      await controller.close();
    });

    test('applies variable deltas and updates arrays across steps', () async {
      final mock = MockRustApiService();
      final controller = StreamController<stream.StepStreamBatch>();

      when(() => mock.compileAndRunMulti(files: any(named: 'files')))
          .thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(() => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')))
          .thenAnswer((_) => controller.stream);
      _stubEmptyVisualization(mock);

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      controller.add(_batch(
        cacheStartStep: 0,
        basePayloads: [
          stream.StepPayloadRef(
            stepIndex: 0,
            codeLine: 1,
            funcNameIdx: 0,
            semanticLabelIdx: 0,
            localVars: [
              stream.ApiVarSnapshotRef(
                nameIdx: 1,
                addr: 0x1000,
                isLocal: true,
                tyNameIdx: 2,
                value: '10',
              ),
            ],
            callStack: [],
            visEvents: [],
            heatmapLine: 0,
            heatmapCount: BigInt.from(0),
            accessedVars: [],
            arraySnapshots: [
              stream.ArraySnapshotRef(
                nameIdx: 3,
                elementTyIdx: 4,
                elements: ['3', '1'],
              ),
            ],
            pointerSnapshots: const [],
          ),
        ],
        deltas: [
          stream.StepPayloadDelta(
            stepIndex: 1,
            codeLine: 2,
            funcNameIdx: 0,
            semanticLabelIdx: 0,
            varDeltas: const [stream.VarDelta(nameIdx: 1, value: '20')],
            newVars: const [],
            removedVarNameIndices: Int32List(0),
            callStack: const [],
            visEvents: const [],
            heatmapLine: 0,
            heatmapCount: BigInt.from(0),
            accessedVars: const [],
            arraySnapshots: const [
              stream.ArraySnapshotRef(
                nameIdx: 3,
                elementTyIdx: 4,
                elements: ['3', '2'],
              ),
            ],
            removedArrayNameIndices: Int32List(0),
            pointerSnapshots: const [],
            removedPointerNameIndices: Int32List(0),
          ),
        ],
        finished: true,
      ));
      await Future.delayed(const Duration(milliseconds: 50));

      final state = container.read(unifiedProvider);
      expect(state.frameCache.length, 2);
      expect(state.frameCache[0].localVars.first.value, '10');
      expect(state.frameCache[1].localVars.first.value, '20');
      expect(state.frameCache[1].arraySnapshots.first.elements, ['3', '2']);
      expect(state.frameCache[1].codeLine, 2);
      expect(state.maxCollectedStep, 1);
      expect(state.currentStep, 1);
      expect(state.currentLine, 2);

      await controller.close();
    });
  });

  group('UnifiedNotifier frame cache window sync', () {
    test('appends payloads when window start does not move', () async {
      final mock = MockRustApiService();
      final controller = StreamController<stream.StepStreamBatch>();

      when(() => mock.compileAndRunMulti(files: any(named: 'files')))
          .thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(() => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')))
          .thenAnswer((_) => controller.stream);
      _stubEmptyVisualization(mock);

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      controller.add(_batchWithSteps(cacheStartStep: 0, startStep: 0, count: 2));
      await Future.delayed(const Duration(milliseconds: 50));

      final state1 = container.read(unifiedProvider);
      expect(state1.frameCacheStartStep, 0);
      expect(state1.frameCache.length, 2);
      expect(state1.frameCache.last.stepIndex, 1);

      controller.add(_batchWithSteps(cacheStartStep: 0, startStep: 2, count: 2));
      await Future.delayed(const Duration(milliseconds: 50));

      final state2 = container.read(unifiedProvider);
      expect(state2.frameCacheStartStep, 0);
      expect(state2.frameCache.length, 4);
      expect(state2.frameCache.last.stepIndex, 3);

      await controller.close();
    });

    test('discards old frames and deduplicates when window slides forward', () async {
      final mock = MockRustApiService();
      final controller = StreamController<stream.StepStreamBatch>();

      when(() => mock.compileAndRunMulti(files: any(named: 'files')))
          .thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(() => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')))
          .thenAnswer((_) => controller.stream);
      _stubEmptyVisualization(mock);

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      controller.add(_batchWithSteps(cacheStartStep: 0, startStep: 0, count: 3));
      await Future.delayed(const Duration(milliseconds: 50));

      final state1 = container.read(unifiedProvider);
      expect(state1.frameCacheStartStep, 0);
      expect(state1.frameCache.length, 3);

      // Second batch starts at step 1 and overlaps step 1..2, then adds step 3.
      controller.add(_batchWithSteps(cacheStartStep: 1, startStep: 1, count: 3));
      await Future.delayed(const Duration(milliseconds: 50));

      final state2 = container.read(unifiedProvider);
      expect(state2.frameCacheStartStep, 1);
      expect(state2.frameCache.length, 3); // [1, 2, 3]
      expect(state2.frameCache.first.stepIndex, 1);
      expect(state2.frameCache.last.stepIndex, 3);

      await controller.close();
    });
  });

  group('UnifiedNotifier memory and heatmap state merge', () {
    test('merges memory regions, fragments and heap stats after batch', () async {
      final mock = MockRustApiService();
      final controller = StreamController<stream.StepStreamBatch>();

      when(() => mock.compileAndRunMulti(files: any(named: 'files')))
          .thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(() => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')))
          .thenAnswer((_) => controller.stream);

      final region = MemoryRegion(
        addr: 0x1000,
        size: 64,
        name: 'x',
        ty: 'int',
        isHeap: true,
        isFreed: false,
        allocLine: 5,
        allocBy: 'malloc',
      );
      final fragment = MemoryFragment(addr: 0x2000, size: 128);
      final stats = HeapStats(totalHeap: 1024, allocated: 64, fragmented: 128, fragmentationRate: 18);

      when(() => mock.getMemoryRegions()).thenAnswer((_) async => [region]);
      when(() => mock.getMemorySize()).thenAnswer((_) async => 1024 * 1024);
      when(() => mock.getMemoryFragments()).thenAnswer((_) async => [fragment]);
      when(() => mock.getHeapStats()).thenAnswer((_) async => stats);
      when(() => mock.getHeatmap()).thenAnswer(
        (_) async => types.HeatmapData(lineCounts: const [], maxCount: BigInt.from(0)),
      );

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      controller.add(_batchWithSteps(cacheStartStep: 0, startStep: 0, count: 1, finished: true));
      await Future.delayed(const Duration(milliseconds: 50));

      final state = container.read(unifiedProvider);
      expect(state.memoryRegions.length, 1);
      expect(state.memoryRegions.first.name, 'x');
      expect(state.memoryFragments.length, 1);
      expect(state.memoryFragments.first.addr, 0x2000);
      expect(state.heapStats?.allocated, 64);
      expect(state.heapStats?.fragmentationRate, 18);

      await controller.close();
    });

    test('finishes with heatmap and correct total steps', () async {
      final mock = MockRustApiService();
      final controller = StreamController<stream.StepStreamBatch>();

      when(() => mock.compileAndRunMulti(files: any(named: 'files')))
          .thenAnswer((_) async => unifiedRunSuccess());
      when(() => mock.getAlgorithmMatches()).thenAnswer((_) async => const []);
      when(() => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')))
          .thenAnswer((_) => controller.stream);
      _stubEmptyVisualization(mock);

      final heatmap = types.HeatmapData(
        lineCounts: [(1, BigInt.from(5))],
        maxCount: BigInt.from(5),
      );
      when(() => mock.getHeatmap()).thenAnswer((_) async => heatmap);

      final container = _createContainer(mock);
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      await notifier.compileAndRun('int main() {}');

      controller.add(_batchWithSteps(cacheStartStep: 0, startStep: 0, count: 4, finished: true));
      await Future.delayed(const Duration(milliseconds: 100));

      final state = container.read(unifiedProvider);
      expect(state.phase, ExecutionPhase.playback);
      expect(state.isPlaying, isFalse);
      expect(state.totalSteps, 3);
      expect(state.heatmap, heatmap);

      await controller.close();
    });
  });

  group('UnifiedNotifier currentVariables helper', () {
    test('returns local vars at current step and empty outside cache', () async {
      final container = _createContainer(MockRustApiService());
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      notifier.state = notifier.state.copyWith(
        phase: ExecutionPhase.playback,
        frameCache: [
          types.StepPayload(
            stepIndex: 0,
            codeLine: 1,
            funcName: 'main',
            semanticLabel: '',
            localVars: [
              types.ApiVariableSnapshot(
                name: 'x',
                value: '10',
                tyName: 'int',
                addr: 0x1000,
                isLocal: true,
              ),
            ],
            callStack: const [],
            visEvents: const [],
            heatmapLine: 0,
            heatmapCount: BigInt.from(0),
            accessedVars: const [],
            arraySnapshots: const [],
            pointerSnapshots: const [],
          ),
        ],
        frameCacheStartStep: 0,
        currentStep: 0,
        maxCollectedStep: 0,
      );

      expect(notifier.currentVariables.length, 1);
      expect(notifier.currentVariables.first.name, 'x');

      notifier.state = notifier.state.copyWith(currentStep: 5);
      expect(notifier.currentVariables, isEmpty);
    });
  });
}

/// Helper to create a batch with consecutive base+delta payloads.
stream.StepStreamBatch _batchWithSteps({
  required int cacheStartStep,
  required int startStep,
  required int count,
  bool finished = false,
}) {
  final base = stream.StepPayloadRef(
    stepIndex: startStep,
    codeLine: startStep + 1,
    funcNameIdx: 0,
    semanticLabelIdx: 0,
    localVars: const [
      stream.ApiVarSnapshotRef(
        nameIdx: 1,
        addr: 0x1000,
        isLocal: true,
        tyNameIdx: 2,
        value: '0',
      ),
    ],
    callStack: const [],
    visEvents: const [],
    heatmapLine: 0,
    heatmapCount: BigInt.from(0),
    accessedVars: const [],
    arraySnapshots: const [],
    pointerSnapshots: const [],
  );

  final deltas = <stream.StepPayloadDelta>[];
  for (var i = 1; i < count; i++) {
    deltas.add(
      stream.StepPayloadDelta(
        stepIndex: startStep + i,
        codeLine: startStep + i + 1,
        funcNameIdx: 0,
        semanticLabelIdx: 0,
        varDeltas: const [stream.VarDelta(nameIdx: 1, value: '1')],
        newVars: const [],
        removedVarNameIndices: Int32List(0),
        callStack: const [],
        visEvents: const [],
        heatmapLine: 0,
        heatmapCount: BigInt.from(0),
        accessedVars: const [],
        arraySnapshots: const [],
        removedArrayNameIndices: Int32List(0),
        pointerSnapshots: const [],
        removedPointerNameIndices: Int32List(0),
      ),
    );
  }

  return _batch(
    cacheStartStep: cacheStartStep,
    basePayloads: [base],
    deltas: deltas,
    finished: finished,
  );
}
