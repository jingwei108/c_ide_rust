import 'dart:async';
import 'package:cide/services/rust_api_service.dart';
import 'package:cide/src/rust/compiler/intent.dart';
import 'package:cide/src/rust/session.dart';
import 'package:cide/src/rust/unified/stream.dart' as stream;
import 'package:cide/src/rust/unified/types.dart' as types;
import 'package:mocktail/mocktail.dart';

class MockRustApiService extends Mock implements RustApiService {}

/// 构造一个编译成功的 [CompileResult]。
CompileResult compileSuccess({List<Diagnostic>? diagnostics}) {
  return CompileResult(
    success: true,
    diagnostics: diagnostics ?? const [],
    algorithmMatches: const [],
  );
}

/// 构造一个编译失败的 [CompileResult]。
CompileResult compileFailure({required String message, int errorCode = 9999}) {
  return CompileResult(
    success: false,
    diagnostics: [
      Diagnostic(
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
  );
}

/// 构造一个运行成功的 [RunResult]。
RunResult runSuccess({String output = ''}) {
  return RunResult(
    success: true,
    output: output,
    waitingInput: false,
    error: null,
  );
}

/// 构造一个运行失败的 [RunResult]。
RunResult runFailure({required String error}) {
  return RunResult(
    success: false,
    output: '',
    waitingInput: false,
    error: error,
  );
}

/// 构造一个单步结果 [StepResult]。
StepResult stepResult({
  StepStatus status = StepStatus.finished,
  int currentLine = 0,
  String output = '',
  bool waitingInput = false,
}) {
  return StepResult(
    status: status,
    currentLine: currentLine,
    output: output,
    waitingInput: waitingInput,
  );
}

/// 构造一个统一模式启动结果 [types.UnifiedRunResult]。
types.UnifiedRunResult unifiedRunSuccess() {
  return const types.UnifiedRunResult(
    success: true,
    error: null,
    totalSteps: 0,
    finished: false,
  );
}

/// 构造一个统一模式启动失败结果 [types.UnifiedRunResult]。
types.UnifiedRunResult unifiedRunFailure({required String error}) {
  return types.UnifiedRunResult(
    success: false,
    error: error,
    totalSteps: 0,
    finished: false,
  );
}

/// 构造一个最简 [types.StepPayload]。
types.StepPayload dummyStepPayload({
  required int stepIndex,
  required int codeLine,
}) {
  return types.StepPayload(
    stepIndex: stepIndex,
    codeLine: codeLine,
    funcName: 'main',
    semanticLabel: '',
    localVars: const [],
    callStack: const [],
    visEvents: const [],
    heatmapLine: 0,
    heatmapCount: BigInt.zero,
    accessedVars: const [],
    arraySnapshots: const [],
    pointerSnapshots: const [],
  );
}

/// 构造一个空的 [stream.StepStreamBatch]。
stream.StepStreamBatch emptyBatch({
  bool finished = false,
  bool trapped = false,
  bool waitingInput = false,
  bool paused = false,
  String? trapMessage,
}) {
  return stream.StepStreamBatch(
    symbolTable: const ['main'],
    basePayloads: const [],
    deltas: const [],
    finished: finished,
    trapped: trapped,
    waitingInput: waitingInput,
    paused: paused,
    currentLine: 0,
    trapMessage: trapMessage,
  );
}
