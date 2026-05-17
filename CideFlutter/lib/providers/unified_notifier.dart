import 'dart:async';
import 'dart:math' as math;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import '../models/unified_state.dart';
import 'ide_provider.dart';

class UnifiedNotifier extends Notifier<UnifiedState> {
  Timer? _playbackTimer;

  @override
  UnifiedState build() {
    return const UnifiedState();
  }

  // ========== 编译与启动 ==========

  Future<void> compileAndRun(String source) async {
    state = state.copyWith(phase: ExecutionPhase.compiling, clearError: true);

    try {
      final result = await rust.compileAndRun(source: source);
      if (!result.success) {
        state = state.copyWith(
          phase: ExecutionPhase.error,
          errorMessage: result.error,
        );
        return;
      }

      // 获取算法检测信息
      final algoMatches = await rust.getAlgorithmMatches();

      state = state.copyWith(
        phase: ExecutionPhase.collecting,
        isPlaying: true,
        currentStep: 0,
        maxCollectedStep: 0,
        totalSteps: 0,
        frameCache: const [],
        algorithmMatches: algoMatches,
        clearError: true,
      );

      _startCollectionLoop();
    } catch (e) {
      state = state.copyWith(
        phase: ExecutionPhase.error,
        errorMessage: '启动失败: $e',
      );
    }
  }

  // ========== 自动收集循环 ==========

  void _startCollectionLoop() {
    _playbackTimer?.cancel();
    // 播放速度映射：1.0x → 50ms, 0.5x → 100ms, 2.0x → 25ms
    final intervalMs = (50 / state.playbackSpeed).round().clamp(16, 500);
    _playbackTimer = Timer.periodic(
      Duration(milliseconds: intervalMs),
      (_) => _collectBatch(),
    );
  }

  Future<void> _collectBatch() async {
    if (state.phase != ExecutionPhase.collecting) {
      return;
    }

    try {
      final result = await rust.runAutoSteps(batchSize: 5);

      if (result.payloads.isNotEmpty) {
        final newCache = [...state.frameCache, ...result.payloads];
        final lastPayload = result.payloads.last;
        state = state.copyWith(
          frameCache: newCache,
          maxCollectedStep: lastPayload.stepIndex,
          currentStep: lastPayload.stepIndex,
          currentLine: lastPayload.codeLine,
        );
      }

      if (result.finished || result.trapped || result.waitingInput) {
        _playbackTimer?.cancel();
        final heatmap = await rust.getHeatmap();
        state = state.copyWith(
          phase: result.trapped ? ExecutionPhase.error : ExecutionPhase.playback,
          isPlaying: false,
          totalSteps: state.maxCollectedStep,
          heatmap: heatmap,
          errorMessage: result.trapped ? (result.trapMessage ?? '运行时错误') : null,
          trapMessage: result.trapped ? (result.trapMessage ?? '运行时错误') : null,
          clearTrap: !result.trapped,
        );
        // 记录学习进度
        await ref.read(ideProvider.notifier).recordUnifiedRun(
          steps: state.maxCollectedStep,
          trapped: result.trapped,
        );
      }
    } catch (e) {
      _playbackTimer?.cancel();
      state = state.copyWith(
        phase: ExecutionPhase.error,
        isPlaying: false,
        errorMessage: '执行异常: $e',
      );
    }
  }

  bool get isPaused => state.phase == ExecutionPhase.paused;

  // ========== 播放控制 ==========

  void pause() {
    if (state.phase == ExecutionPhase.collecting) {
      rust.pauseExecution();
      _playbackTimer?.cancel();
      state = state.copyWith(phase: ExecutionPhase.paused, isPlaying: false);
    }
  }

  void resume() {
    if (state.phase == ExecutionPhase.playback) {
      _continueFromPlayback();
      return;
    }
    if (state.phase == ExecutionPhase.paused || state.phase == ExecutionPhase.stepMode) {
      rust.resumeExecution();
      state = state.copyWith(phase: ExecutionPhase.collecting, isPlaying: true);
      _startCollectionLoop();
    }
  }

  Future<void> _continueFromPlayback() async {
    state = state.copyWith(phase: ExecutionPhase.seeking);
    try {
      await rust.seekToStep(target: state.currentStep);
      state = state.copyWith(isVmRestored: true);
      rust.resumeExecution();
      state = state.copyWith(phase: ExecutionPhase.collecting, isPlaying: true);
      _startCollectionLoop();
    } catch (e) {
      state = state.copyWith(
        phase: ExecutionPhase.error,
        errorMessage: '恢复执行失败: $e',
      );
    }
  }

  // ========== 单步 ==========

  Future<void> stepNext() async {
    if (state.phase == ExecutionPhase.idle || state.phase == ExecutionPhase.compiling) {
      return;
    }

    state = state.copyWith(phase: ExecutionPhase.stepMode, isPlaying: false);
    _playbackTimer?.cancel();

    try {
      final payload = await rust.stepNextUnified();
      if (payload != null) {
        final newCache = [...state.frameCache];
        if (payload.stepIndex < newCache.length) {
          newCache[payload.stepIndex] = payload;
        } else {
          newCache.add(payload);
        }
        state = state.copyWith(
          frameCache: newCache,
          currentStep: payload.stepIndex,
          currentLine: payload.codeLine,
          maxCollectedStep: math.max(state.maxCollectedStep, payload.stepIndex),
          isVmRestored: true,
        );
      }
    } catch (e) {
      state = state.copyWith(errorMessage: '单步异常: $e');
    }
  }

  // ========== Seek / 拖动进度条 ==========

  Future<void> seekTo(int targetStep) async {
    if (!state.canSeek) return;

    // 即时响应：目标已在缓存中
    if (targetStep >= 0 && targetStep <= state.maxCollectedStep && targetStep < state.frameCache.length) {
      final payload = state.frameCache[targetStep];
      state = state.copyWith(
        currentStep: targetStep,
        currentLine: payload.codeLine,
        phase: ExecutionPhase.playback,
      );
      return;
    }

    // 需要后台恢复 VM
    state = state.copyWith(phase: ExecutionPhase.seeking);
    // 记录 Seek 操作
    await ref.read(ideProvider.notifier).recordSeek();
    try {
      final result = await rust.seekToStep(target: targetStep);
      if (result.success && result.payload != null) {
        final payload = result.payload!;
        final newCache = [...state.frameCache];
        if (payload.stepIndex < newCache.length) {
          newCache[payload.stepIndex] = payload;
        } else if (payload.stepIndex == newCache.length) {
          newCache.add(payload);
        }
        state = state.copyWith(
          frameCache: newCache,
          currentStep: targetStep,
          currentLine: payload.codeLine,
          isVmRestored: true,
          phase: ExecutionPhase.playback,
          maxCollectedStep: math.max(state.maxCollectedStep, targetStep),
        );
      } else {
        state = state.copyWith(
          phase: ExecutionPhase.error,
          errorMessage: result.error ?? 'Seek 失败',
        );
      }
    } catch (e) {
      state = state.copyWith(
        phase: ExecutionPhase.error,
        errorMessage: 'Seek 异常: $e',
      );
    }
  }

  void onSliderChanged(int targetStep) {
    // 拖动过程中即时更新（不恢复 VM）
    if (targetStep >= 0 && targetStep < state.frameCache.length) {
      final payload = state.frameCache[targetStep];
      state = state.copyWith(
        currentStep: targetStep,
        currentLine: payload.codeLine,
      );
    }
  }

  // ========== 播放速度 ==========

  void setPlaybackSpeed(double speed) {
    state = state.copyWith(playbackSpeed: speed);
    if (state.phase == ExecutionPhase.collecting) {
      _startCollectionLoop(); // 重启 Timer 以应用新速度
    }
  }

  // ========== 重置 ==========

  void reset() {
    _playbackTimer?.cancel();
    rust.resetSession();
    state = const UnifiedState();
  }

  void onCodeChanged() {
    if (state.phase != ExecutionPhase.idle) {
      _playbackTimer?.cancel();
      state = state.copyWith(
        phase: ExecutionPhase.idle,
        isPlaying: false,
        frameCache: const [],
        currentStep: 0,
        maxCollectedStep: 0,
        totalSteps: 0,
        algorithmMatches: const [],
      );
    }
  }
}
