import 'package:cide/models/ide_state.dart';
import 'package:cide/models/unified_state.dart';
import 'package:cide/src/rust/api/types.dart' as rust;
import 'package:cide/src/rust/session.dart' as rust_session;
import 'package:cide/src/rust/unified/types.dart' as unified;

/// Factory helpers for constructing common [IdeState] and [UnifiedState]
/// scenarios in widget/provider tests without touching real Rust APIs.

/// A minimal [IdeState] ready for compilation/running tests.
IdeState idleIdeState() => const IdeState();

/// [IdeState] with a compilation error diagnostic.
IdeState ideStateWithDiagnostic(rust.Diagnostic diagnostic) {
  return IdeState(diagnostics: [diagnostic], output: '编译失败');
}

/// [IdeState] while waiting for stdin input during a run.
IdeState ideStateWaitingInput() {
  return const IdeState(isRunning: true, waitingInput: true, output: 'prompt');
}

/// [IdeState] with a non-empty output.
IdeState ideStateWithOutput(String output) {
  return IdeState(output: output);
}

/// [IdeState] with watch expressions and algorithm matches.
IdeState ideStateWithMatches(List<rust_session.AlgorithmMatch> matches) {
  return IdeState(algorithmMatches: matches);
}

/// A minimal [UnifiedState] in playback phase with the given cache.
UnifiedState playbackState(List<unified.StepPayload> cache) {
  return UnifiedState(
    phase: ExecutionPhase.playback,
    frameCache: cache,
    maxCollectedStep: cache.isEmpty ? 0 : cache.last.stepIndex,
  );
}

/// [UnifiedState] in playback phase with memory visualization data.
UnifiedState playbackStateWithMemory({
  required List<unified.StepPayload> cache,
  required List<rust_session.MemoryRegion> regions,
  List<rust_session.MemoryFragment> fragments = const [],
  rust_session.HeapStats? heapStats,
  int memorySize = 1024 * 1024,
}) {
  return UnifiedState(
    phase: ExecutionPhase.playback,
    frameCache: cache,
    maxCollectedStep: cache.isEmpty ? 0 : cache.last.stepIndex,
    memoryRegions: regions,
    memoryFragments: fragments,
    heapStats: heapStats,
    memorySize: memorySize,
  );
}

/// [UnifiedState] in collecting phase.
UnifiedState collectingState() {
  return const UnifiedState(phase: ExecutionPhase.collecting, isPlaying: true);
}

/// [UnifiedState] paused with a frame cache.
UnifiedState pausedState(List<unified.StepPayload> cache) {
  return UnifiedState(
    phase: ExecutionPhase.paused,
    frameCache: cache,
    maxCollectedStep: cache.isEmpty ? 0 : cache.last.stepIndex,
  );
}

/// Construct a [unified.StepPayload] with the given local vars and call stack.
unified.StepPayload stepPayload({
  required int stepIndex,
  int codeLine = 0,
  String funcName = 'main',
  String semanticLabel = '',
  List<unified.ApiVariableSnapshot> localVars = const [],
  List<unified.ApiFrameInfo> callStack = const [],
  List<unified.AccessedVar> accessedVars = const [],
  List<unified.ArraySnapshot> arraySnapshots = const [],
  List<unified.PointerSnapshot> pointerSnapshots = const [],
  List<rust_session.VisEvent> visEvents = const [],
  unified.AlgorithmStepSnapshot? algorithmStep,
}) {
  return unified.StepPayload(
    stepIndex: stepIndex,
    codeLine: codeLine,
    funcName: funcName,
    semanticLabel: semanticLabel,
    localVars: localVars,
    callStack: callStack,
    visEvents: visEvents,
    heatmapLine: 0,
    heatmapCount: BigInt.zero,
    accessedVars: accessedVars,
    arraySnapshots: arraySnapshots,
    pointerSnapshots: pointerSnapshots,
    algorithmStep: algorithmStep,
  );
}

/// Construct a local variable snapshot.
unified.ApiVariableSnapshot variable({
  required String name,
  required String value,
  String tyName = 'int',
  int addr = 0x1000,
  bool isLocal = true,
}) {
  return unified.ApiVariableSnapshot(
    name: name,
    value: value,
    tyName: tyName,
    addr: addr,
    isLocal: isLocal,
  );
}

/// Construct a call stack frame.
unified.ApiFrameInfo frame({
  String funcName = 'main',
  int returnLine = 0,
}) {
  return unified.ApiFrameInfo(funcName: funcName, returnLine: returnLine);
}

/// Construct an algorithm match.
rust_session.AlgorithmMatch algorithmMatch({
  String name = 'bubble_sort',
  String displayName = '冒泡排序',
  int confidence = 95,
  List<rust_session.VisEvent> visEvents = const [],
}) {
  return rust_session.AlgorithmMatch(
    name: name,
    displayName: displayName,
    funcName: 'main',
    confidence: confidence,
    suggestion: '',
    line: 1,
    visEvents: visEvents,
  );
}

/// Construct a diagnostic.
rust_session.Diagnostic diagnostic({
  String message = 'error',
  int line = 1,
  int errorCode = 1001,
  int severity = 0,
  String fixSuggestion = '',
  int fixKind = 0,
}) {
  return rust_session.Diagnostic(
    message: message,
    line: line,
    column: 1,
    errorCode: errorCode,
    severity: severity,
    fixSuggestion: fixSuggestion,
    fixKind: fixKind,
    replaceStartLine: 0,
    replaceStartColumn: 0,
    replaceEndLine: 0,
    replaceEndColumn: 0,
    replacementText: '',
    filename: 'main.c',
  );
}

/// Construct an array snapshot.
unified.ArraySnapshot arraySnapshot({
  String name = 'arr',
  String elementTy = 'int',
  List<String> elements = const [],
}) {
  return unified.ArraySnapshot(
    name: name,
    elementTy: elementTy,
    elements: elements,
  );
}

/// Construct a pointer snapshot.
unified.PointerSnapshot pointerSnapshot({
  String name = 'p',
  String tyName = 'int*',
  int addr = 0x2000,
  int targetAddr = 0x3000,
  String targetName = 'x',
  unified.PointerStatus status = unified.PointerStatus.valid,
}) {
  return unified.PointerSnapshot(
    name: name,
    tyName: tyName,
    addr: addr,
    targetAddr: targetAddr,
    targetName: targetName,
    status: status,
  );
}

/// Construct a memory region.
rust_session.MemoryRegion memoryRegion({
  int addr = 0x1000,
  int size = 64,
  String name = 'x',
  String ty = 'int',
  bool isHeap = false,
  bool isFreed = false,
  int allocLine = 0,
  String allocBy = '',
}) {
  return rust_session.MemoryRegion(
    addr: addr,
    size: size,
    name: name,
    ty: ty,
    isHeap: isHeap,
    isFreed: isFreed,
    allocLine: allocLine,
    allocBy: allocBy,
  );
}

/// Construct a memory fragment.
rust_session.MemoryFragment memoryFragment({
  int addr = 0x2000,
  int size = 128,
}) {
  return rust_session.MemoryFragment(addr: addr, size: size);
}

/// Construct heap statistics.
rust_session.HeapStats heapStats({
  int totalHeap = 1024,
  int allocated = 256,
  int fragmented = 128,
  int fragmentationRate = 37,
}) {
  return rust_session.HeapStats(
    totalHeap: totalHeap,
    allocated: allocated,
    fragmented: fragmented,
    fragmentationRate: fragmentationRate,
  );
}

/// Construct a visualization event for [unified.StepPayload].
rust_session.VisEvent visEvent({
  int ty = 0,
  int line = 1,
  int extra0 = 0,
  int extra1 = 0,
  int extra2 = 0,
  String context = '',
}) {
  return rust_session.VisEvent(
    ty: ty,
    line: line,
    extra0: extra0,
    extra1: extra1,
    extra2: extra2,
    context: context,
  );
}
