import 'dart:async';
import 'package:mocktail/mocktail.dart';
import 'package:cide/src/rust/session.dart' as rust_session;
import 'package:cide/src/rust/unified/stream.dart' as stream;
import 'package:cide/src/rust/unified/types.dart' as unified;
import '../mocks/rust_api_service_mock.dart';

/// Pre-configured stubs for [MockRustApiService] covering the most common
/// frontend functional test scenarios.

/// Stub a successful compile with no diagnostics.
void stubCompileSuccess(MockRustApiService mock) {
  when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
    (_) async => const rust_session.CompileResult(
      success: true,
      diagnostics: [],
      algorithmMatches: [],
    ),
  );
  when(
    () => mock.inferIntentFromSource(source: any(named: 'source')),
  ).thenAnswer((_) async => const []);
}

/// Stub a compile failure with a single diagnostic.
void stubCompileFailure(
  MockRustApiService mock, {
  String message = '编译错误',
  int errorCode = 9999,
}) {
  when(() => mock.compileMulti(files: any(named: 'files'))).thenAnswer(
    (_) async => rust_session.CompileResult(
      success: false,
      diagnostics: [
        rust_session.Diagnostic(
          line: 1,
          column: 1,
          errorCode: errorCode,
          severity: 0,
          message: message,
          fixSuggestion: '',
          fixKind: 0,
          replaceStartLine: 0,
          replaceStartColumn: 0,
          replaceEndLine: 0,
          replaceEndColumn: 0,
          replacementText: '',
          filename: 'main.c',
        ),
      ],
      algorithmMatches: const [],
    ),
  );
}

/// Stub a successful run that produces [output].
void stubRunSuccess(MockRustApiService mock, {String output = ''}) {
  when(() => mock.runCode()).thenAnswer(
    (_) async => rust_session.RunResult(
      success: true,
      output: output,
      waitingInput: false,
      error: null,
    ),
  );
}

/// Stub a run failure with the given error message.
void stubRunFailure(MockRustApiService mock, {required String error}) {
  when(() => mock.runCode()).thenAnswer(
    (_) async => rust_session.RunResult(
      success: false,
      output: '',
      waitingInput: false,
      error: error,
    ),
  );
}

/// Stub the full unified-mode pipeline to use a controlled [StreamController].
///
/// Returns the controller so the test can push batches. The stream batch type
/// uses symbol-table references ([stream.StepPayloadRef]); tests that need to
/// feed frames should construct those directly or use an empty finished batch.
StreamController<stream.StepStreamBatch> stubUnifiedPipeline(
  MockRustApiService mock,
) {
  final controller = StreamController<stream.StepStreamBatch>();

  when(
    () => mock.compileAndRunMulti(files: any(named: 'files')),
  ).thenAnswer(
    (_) async => const unified.UnifiedRunResult(
      success: true,
      error: null,
      totalSteps: 0,
      finished: false,
    ),
  );
  when(() => mock.getAlgorithmMatches()).thenAnswer(
    (_) async => const <rust_session.AlgorithmMatch>[],
  );
  when(
    () => mock.runAutoStepsStream(batchSize: any(named: 'batchSize')),
  ).thenAnswer((_) => controller.stream);
  stubEmptyVisualization(mock);
  when(() => mock.getHeatmap()).thenAnswer(
    (_) async => unified.HeatmapData(
      lineCounts: const [],
      maxCount: BigInt.from(0),
    ),
  );

  return controller;
}

/// Stub all memory/visualization related APIs to return empty defaults.
void stubEmptyVisualization(MockRustApiService mock) {
  when(() => mock.getMemoryRegions()).thenAnswer(
    (_) async => const <rust_session.MemoryRegion>[],
  );
  when(() => mock.getMemorySize()).thenAnswer((_) async => 1024 * 1024);
  when(() => mock.getMemoryFragments()).thenAnswer(
    (_) async => const <rust_session.MemoryFragment>[],
  );
  when(() => mock.getHeapStats()).thenAnswer(
    (_) async => const rust_session.HeapStats(
      totalHeap: 0,
      allocated: 0,
      fragmented: 0,
      fragmentationRate: 0,
    ),
  );
}
