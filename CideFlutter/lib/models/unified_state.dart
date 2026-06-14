import 'package:cide/src/rust/session.dart' as rust_session;
import 'package:cide/src/rust/unified/types.dart' as rust;

enum ExecutionPhase {
  idle,
  compiling,
  collecting,
  paused,
  playback,
  seeking,
  stepMode,
  error,
}

class UnifiedState {
  final ExecutionPhase phase;
  final int currentStep;
  final int maxCollectedStep;
  final int totalSteps;
  final List<rust.StepPayload> frameCache;
  final bool isPlaying;
  final double playbackSpeed;
  final String? errorMessage;
  final bool isVmRestored;
  final rust.HeatmapData? heatmap;
  final int currentLine;
  final String? trapMessage;
  final List<rust_session.AlgorithmMatch> algorithmMatches;

  // 统一可视化数据源：内存与变量快照由 UnifiedNotifier 维护，
  // 各可视化 Tab 直接从这里读取，避免重复调用 Rust FFI。
  final List<rust_session.MemoryRegion> memoryRegions;
  final int memorySize;
  final List<rust_session.MemoryFragment> memoryFragments;
  final rust_session.HeapStats? heapStats;

  const UnifiedState({
    this.phase = ExecutionPhase.idle,
    this.currentStep = 0,
    this.maxCollectedStep = 0,
    this.totalSteps = 0,
    this.frameCache = const [],
    this.isPlaying = false,
    this.playbackSpeed = 1.0,
    this.errorMessage,
    this.isVmRestored = false,
    this.heatmap,
    this.currentLine = 0,
    this.trapMessage,
    this.algorithmMatches = const [],
    this.memoryRegions = const [],
    this.memorySize = 1024 * 1024,
    this.memoryFragments = const [],
    this.heapStats,
  });

  UnifiedState copyWith({
    ExecutionPhase? phase,
    int? currentStep,
    int? maxCollectedStep,
    int? totalSteps,
    List<rust.StepPayload>? frameCache,
    bool? isPlaying,
    double? playbackSpeed,
    String? errorMessage,
    bool? isVmRestored,
    rust.HeatmapData? heatmap,
    int? currentLine,
    String? trapMessage,
    List<rust_session.AlgorithmMatch>? algorithmMatches,
    List<rust_session.MemoryRegion>? memoryRegions,
    int? memorySize,
    List<rust_session.MemoryFragment>? memoryFragments,
    rust_session.HeapStats? heapStats,
    bool clearError = false,
    bool clearTrap = false,
  }) {
    return UnifiedState(
      phase: phase ?? this.phase,
      currentStep: currentStep ?? this.currentStep,
      maxCollectedStep: maxCollectedStep ?? this.maxCollectedStep,
      totalSteps: totalSteps ?? this.totalSteps,
      frameCache: frameCache ?? this.frameCache,
      isPlaying: isPlaying ?? this.isPlaying,
      playbackSpeed: playbackSpeed ?? this.playbackSpeed,
      errorMessage: clearError ? null : (errorMessage ?? this.errorMessage),
      isVmRestored: isVmRestored ?? this.isVmRestored,
      heatmap: heatmap ?? this.heatmap,
      currentLine: currentLine ?? this.currentLine,
      trapMessage: clearTrap ? null : (trapMessage ?? this.trapMessage),
      algorithmMatches: algorithmMatches ?? this.algorithmMatches,
      memoryRegions: memoryRegions ?? this.memoryRegions,
      memorySize: memorySize ?? this.memorySize,
      memoryFragments: memoryFragments ?? this.memoryFragments,
      heapStats: heapStats ?? this.heapStats,
    );
  }

  bool get canPlay =>
      phase == ExecutionPhase.idle ||
      phase == ExecutionPhase.paused ||
      phase == ExecutionPhase.playback ||
      phase == ExecutionPhase.stepMode;

  bool get canPause => phase == ExecutionPhase.collecting;

  bool get canStep =>
      phase == ExecutionPhase.paused || phase == ExecutionPhase.stepMode;

  bool get canSeek =>
      phase == ExecutionPhase.playback ||
      phase == ExecutionPhase.paused ||
      phase == ExecutionPhase.stepMode ||
      phase == ExecutionPhase.collecting;

  bool get showSlider =>
      phase != ExecutionPhase.idle &&
      phase != ExecutionPhase.compiling &&
      phase != ExecutionPhase.error;
}
