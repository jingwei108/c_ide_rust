import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import 'package:cide/src/rust/compiler/intent.dart';
import 'package:cide/src/rust/session.dart';
import 'package:cide/src/rust/unified/stream.dart' as stream;
import 'package:cide/src/rust/unified/types.dart' as types;
import '../models/ide_state.dart' show CodeFile;

/// Rust API 服务的全局 Provider。
///
/// 默认使用 [DefaultRustApiService] 连接真实 Rust 后端；
/// 单元测试中可通过 [ProviderContainer] 的 `overrides` 注入 mock 实现。
final rustApiServiceProvider = Provider<RustApiService>(
  (ref) => DefaultRustApiService(),
);

/// Rust 后端 API 的抽象层。
///
/// 所有与 `flutter_rust_bridge` 生成的全局函数交互的逻辑都通过此接口，
/// 便于在单元测试中用 mock 替换，避免在 Dart VM 中加载原生动态库。
abstract class RustApiService {
  Future<CompileResult> compileMulti({required List<CodeFile> files});
  Future<RunResult> runCode();
  Future<StepResult> stepNext();
  Future<void> provideInputLine({required String line});
  Future<void> resetSession();
  Future<void> setBreakpoints({required List<int> lines});
  Future<List<IntentScore>> inferIntentFromSource({required String source});
  Future<types.UnifiedRunResult> compileAndRunMulti({required List<CodeFile> files});
  Future<List<AlgorithmMatch>> getAlgorithmMatches();
  Stream<stream.StepStreamBatch> runAutoStepsStream({required int batchSize});
  Future<List<MemoryRegion>> getMemoryRegions();
  Future<int> getMemorySize();
  Future<List<MemoryFragment>> getMemoryFragments();
  Future<HeapStats> getHeapStats();
  Future<types.HeatmapData> getHeatmap();
  Future<void> pauseExecution();
  Future<void> resumeExecution();
  Future<types.SeekResult> seekToStep({required int target});
  Future<types.StepPayload?> stepNextUnified();
  Future<int> getFrameCacheStartStep();
  Future<String?> applyFix({required String source, required Diagnostic diag});
  Future<CompileResult> compile({required String source});
}

/// 默认实现：直接转发到 `flutter_rust_bridge` 生成的全局函数。
class DefaultRustApiService implements RustApiService {
  @override
  Future<CompileResult> compileMulti({required List<CodeFile> files}) async {
    return rust.compileMulti(
      files:
          files
              .map((f) => rust.CodeFile(filename: f.filename, source: f.source))
              .toList(),
    );
  }

  @override
  Future<RunResult> runCode() => rust.runCode();

  @override
  Future<StepResult> stepNext() => rust.stepNext();

  @override
  Future<void> provideInputLine({required String line}) =>
      rust.provideInputLine(line: line);

  @override
  Future<void> resetSession() => rust.resetSession();

  @override
  Future<void> setBreakpoints({required List<int> lines}) =>
      rust.setBreakpoints(lines: lines);

  @override
  Future<List<IntentScore>> inferIntentFromSource({required String source}) =>
      rust.inferIntentFromSource(source: source);

  @override
  Future<types.UnifiedRunResult> compileAndRunMulti({required List<CodeFile> files}) async {
    return rust.compileAndRunMulti(
      files:
          files
              .map((f) => rust.CodeFile(filename: f.filename, source: f.source))
              .toList(),
    );
  }

  @override
  Future<List<AlgorithmMatch>> getAlgorithmMatches() =>
      rust.getAlgorithmMatches();

  @override
  Stream<stream.StepStreamBatch> runAutoStepsStream({required int batchSize}) =>
      rust.runAutoStepsStream(batchSize: batchSize);

  @override
  Future<List<MemoryRegion>> getMemoryRegions() => rust.getMemoryRegions();

  @override
  Future<int> getMemorySize() => rust.getMemorySize();

  @override
  Future<List<MemoryFragment>> getMemoryFragments() =>
      rust.getMemoryFragments();

  @override
  Future<HeapStats> getHeapStats() => rust.getHeapStats();

  @override
  Future<types.HeatmapData> getHeatmap() => rust.getHeatmap();

  @override
  Future<void> pauseExecution() => rust.pauseExecution();

  @override
  Future<void> resumeExecution() => rust.resumeExecution();

  @override
  Future<types.SeekResult> seekToStep({required int target}) =>
      rust.seekToStep(target: target);

  @override
  Future<types.StepPayload?> stepNextUnified() => rust.stepNextUnified();

  @override
  Future<int> getFrameCacheStartStep() => rust.getFrameCacheStartStep();

  @override
  Future<String?> applyFix({required String source, required Diagnostic diag}) =>
      rust.applyFix(source: source, diag: diag);

  @override
  Future<CompileResult> compile({required String source}) =>
      rust.compile(source: source);
}

