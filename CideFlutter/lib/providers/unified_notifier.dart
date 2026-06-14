import 'dart:async';
import 'dart:math' as math;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import 'package:cide/src/rust/session.dart';
import 'package:cide/src/rust/unified/stream.dart' as stream;
import 'package:cide/src/rust/unified/types.dart' as types;
import '../models/unified_state.dart';
import 'ide_provider.dart';

class UnifiedNotifier extends AutoDisposeNotifier<UnifiedState> {
  StreamSubscription<stream.StepStreamBatch>? _streamSubscription;

  @override
  UnifiedState build() {
    ref.onDispose(() {
      _streamSubscription?.cancel();
    });
    return const UnifiedState();
  }

  // ========== 编译与启动 ==========

  Future<void> compileAndRun(String source) async {
    await compileAndRunMulti([CodeFile(filename: 'main.c', source: source)]);
  }

  Future<void> compileAndRunMulti(List<CodeFile> files) async {
    state = state.copyWith(phase: ExecutionPhase.compiling, clearError: true);

    try {
      final result = await rust.compileAndRunMulti(
        files:
            files
                .map(
                  (f) => rust.CodeFile(filename: f.filename, source: f.source),
                )
                .toList(),
      );
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
    _streamSubscription?.cancel();

    final stream = rust.runAutoStepsStream(batchSize: 100);
    _streamSubscription = stream.listen(
      _onBatchReceived,
      onError: (Object e) {
        state = state.copyWith(
          phase: ExecutionPhase.error,
          isPlaying: false,
          errorMessage: '执行异常: $e',
        );
      },
    );
  }

  void _onBatchReceived(stream.StepStreamBatch batch) {
    if (state.phase != ExecutionPhase.collecting) {
      return;
    }

    final payloads = _decodeBatch(batch);
    if (payloads.isNotEmpty) {
      final newCache = [...state.frameCache, ...payloads];
      final lastPayload = payloads.last;
      state = state.copyWith(
        frameCache: newCache,
        maxCollectedStep: lastPayload.stepIndex,
        currentStep: lastPayload.stepIndex,
        currentLine: lastPayload.codeLine,
      );
      // 同步更新内存可视化数据源
      _fetchMemoryState();
    }

    if (batch.paused) {
      _streamSubscription?.cancel();
      state = state.copyWith(phase: ExecutionPhase.paused, isPlaying: false);
    } else if (batch.finished || batch.trapped || batch.waitingInput) {
      _streamSubscription?.cancel();
      _fetchHeatmapAndFinish(batch.trapped, batch.trapMessage);
    }
  }

  /// 从 Rust VM 拉取当前内存状态，供 MemoryTab 等可视化组件统一读取。
  Future<void> _fetchMemoryState() async {
    try {
      final results = await Future.wait([
        rust.getMemoryRegions(),
        rust.getMemorySize(),
        rust.getMemoryFragments(),
        rust.getHeapStats(),
      ]);
      state = state.copyWith(
        memoryRegions: results[0] as List<MemoryRegion>,
        memorySize: results[1] as int,
        memoryFragments: results[2] as List<MemoryFragment>,
        heapStats: results[3] as HeapStats,
      );
    } catch (_) {
      // 内存信息属于辅助可视化数据，失败时不阻塞主执行流程。
    }
  }

  Future<void> _fetchHeatmapAndFinish(bool trapped, String? trapMessage) async {
    final heatmap = await rust.getHeatmap();
    state = state.copyWith(
      phase: trapped ? ExecutionPhase.error : ExecutionPhase.playback,
      isPlaying: false,
      totalSteps: state.maxCollectedStep,
      heatmap: heatmap,
      errorMessage: trapped ? (trapMessage ?? '运行时错误') : null,
      trapMessage: trapped ? (trapMessage ?? '运行时错误') : null,
      clearTrap: !trapped,
    );
    await ref
        .read(ideProvider.notifier)
        .recordUnifiedRun(
          steps: state.maxCollectedStep,
          trapped: trapped,
          trapMessage: trapped ? (trapMessage ?? '运行时错误') : null,
        );
  }

  /// 将 StepStreamBatch 解码为完整的 StepPayload 列表。
  List<types.StepPayload> _decodeBatch(stream.StepStreamBatch batch) {
    if (batch.basePayloads.isEmpty) return [];

    final sym = batch.symbolTable;
    final result = <types.StepPayload>[];

    // 解码 base payload
    result.add(_decodeStepPayloadRef(batch.basePayloads.first, sym));

    // 维护当前变量状态（按 nameIdx 索引）
    final currentVars = <int, types.ApiVariableSnapshot>{};
    for (final v in batch.basePayloads.first.localVars) {
      currentVars[v.nameIdx] = types.ApiVariableSnapshot(
        name: sym[v.nameIdx],
        addr: v.addr,
        isLocal: v.isLocal,
        tyName: sym[v.tyNameIdx],
        value: v.value,
      );
    }

    // 维护变量出现顺序（base 顺序 + 后续新增追加到末尾）
    final varOrder =
        batch.basePayloads.first.localVars.map((v) => v.nameIdx).toList();
    final varOrderSet = <int>{...varOrder};

    for (final delta in batch.deltas) {
      // 应用变化的变量
      for (final d in delta.varDeltas) {
        if (currentVars.containsKey(d.nameIdx)) {
          final old = currentVars[d.nameIdx]!;
          currentVars[d.nameIdx] = types.ApiVariableSnapshot(
            name: old.name,
            addr: old.addr,
            isLocal: old.isLocal,
            tyName: old.tyName,
            value: d.value,
          );
        }
      }

      // 移除消失的变量
      for (final idx in delta.removedVarNameIndices) {
        currentVars.remove(idx);
        varOrderSet.remove(idx);
      }

      // 添加新变量
      for (final v in delta.newVars) {
        currentVars[v.nameIdx] = types.ApiVariableSnapshot(
          name: _safeSym(sym, v.nameIdx),
          addr: v.addr,
          isLocal: v.isLocal,
          tyName: _safeSym(sym, v.tyNameIdx),
          value: v.value,
        );
        if (!varOrderSet.contains(v.nameIdx)) {
          varOrder.add(v.nameIdx);
          varOrderSet.add(v.nameIdx);
        }
      }

      // 按 varOrder 构建 localVars（跳过已移除的）
      final localVars = <types.ApiVariableSnapshot>[];
      for (final nameIdx in varOrder) {
        if (currentVars.containsKey(nameIdx)) {
          localVars.add(currentVars[nameIdx]!);
        }
      }

      result.add(
        types.StepPayload(
          stepIndex: delta.stepIndex,
          codeLine: delta.codeLine,
          funcName: sym[delta.funcNameIdx],
          semanticLabel: sym[delta.semanticLabelIdx],
          algorithmStep:
              delta.algorithmStep == null
                  ? null
                  : types.AlgorithmStepSnapshot(
                    algorithmName: sym[delta.algorithmStep!.algorithmNameIdx],
                    displayName: sym[delta.algorithmStep!.displayNameIdx],
                    phase: sym[delta.algorithmStep!.phaseIdx],
                    description: sym[delta.algorithmStep!.descriptionIdx],
                  ),
          localVars: localVars,
          callStack:
              delta.callStack
                  .map(
                    (f) => types.ApiFrameInfo(
                      funcName: sym[f.funcNameIdx],
                      returnLine: f.returnLine,
                    ),
                  )
                  .toList(),
          visEvents: delta.visEvents,
          heatmapLine: delta.heatmapLine,
          heatmapCount: delta.heatmapCount,
          accessedVars:
              delta.accessedVars
                  .map(
                    (a) => types.AccessedVar(
                      name: sym[a.nameIdx],
                      accessType: sym[a.accessTypeIdx],
                    ),
                  )
                  .toList(),
          arraySnapshots:
              delta.arraySnapshots
                  .map(
                    (a) => types.ArraySnapshot(
                      name: sym[a.nameIdx],
                      elementTy: sym[a.elementTyIdx],
                      elements: a.elements,
                    ),
                  )
                  .toList(),
          pointerSnapshots:
              delta.pointerSnapshots
                  .map(
                    (p) => types.PointerSnapshot(
                      name: sym[p.nameIdx],
                      addr: p.addr,
                      tyName: sym[p.tyNameIdx],
                      targetAddr: p.targetAddr,
                      targetName: sym[p.targetNameIdx],
                      status: p.status,
                    ),
                  )
                  .toList(),
          rootCauseHint: delta.rootCauseHint,
        ),
      );
    }

    return result;
  }

  String _safeSym(List<String> sym, int idx) {
    if (idx >= 0 && idx < sym.length) return sym[idx];
    return '';
  }

  types.StepPayload _decodeStepPayloadRef(
    stream.StepPayloadRef base,
    List<String> sym,
  ) {
    return types.StepPayload(
      stepIndex: base.stepIndex,
      codeLine: base.codeLine,
      funcName: _safeSym(sym, base.funcNameIdx),
      semanticLabel: sym[base.semanticLabelIdx],
      algorithmStep:
          base.algorithmStep == null
              ? null
              : types.AlgorithmStepSnapshot(
                algorithmName: _safeSym(
                  sym,
                  base.algorithmStep!.algorithmNameIdx,
                ),
                displayName: _safeSym(sym, base.algorithmStep!.displayNameIdx),
                phase: _safeSym(sym, base.algorithmStep!.phaseIdx),
                description: _safeSym(sym, base.algorithmStep!.descriptionIdx),
              ),
      localVars:
          base.localVars
              .map(
                (v) => types.ApiVariableSnapshot(
                  name: _safeSym(sym, v.nameIdx),
                  addr: v.addr,
                  isLocal: v.isLocal,
                  tyName: _safeSym(sym, v.tyNameIdx),
                  value: v.value,
                ),
              )
              .toList(),
      callStack:
          base.callStack
              .map(
                (f) => types.ApiFrameInfo(
                  funcName: _safeSym(sym, f.funcNameIdx),
                  returnLine: f.returnLine,
                ),
              )
              .toList(),
      visEvents: base.visEvents,
      heatmapLine: base.heatmapLine,
      heatmapCount: base.heatmapCount,
      accessedVars:
          base.accessedVars
              .map(
                (a) => types.AccessedVar(
                  name: _safeSym(sym, a.nameIdx),
                  accessType: _safeSym(sym, a.accessTypeIdx),
                ),
              )
              .toList(),
      arraySnapshots:
          base.arraySnapshots
              .map(
                (a) => types.ArraySnapshot(
                  name: _safeSym(sym, a.nameIdx),
                  elementTy: _safeSym(sym, a.elementTyIdx),
                  elements: a.elements,
                ),
              )
              .toList(),
      pointerSnapshots:
          base.pointerSnapshots
              .map(
                (p) => types.PointerSnapshot(
                  name: _safeSym(sym, p.nameIdx),
                  addr: p.addr,
                  tyName: _safeSym(sym, p.tyNameIdx),
                  targetAddr: p.targetAddr,
                  targetName: _safeSym(sym, p.targetNameIdx),
                  status: p.status,
                ),
              )
              .toList(),
      rootCauseHint: base.rootCauseHint,
    );
  }

  bool get isPaused => state.phase == ExecutionPhase.paused;

  // ========== 播放控制 ==========

  void pause() {
    if (state.phase == ExecutionPhase.collecting) {
      rust.pauseExecution();
      _streamSubscription?.cancel();
      state = state.copyWith(phase: ExecutionPhase.paused, isPlaying: false);
    }
  }

  void resume() {
    if (state.phase == ExecutionPhase.playback) {
      _continueFromPlayback();
      return;
    }
    if (state.phase == ExecutionPhase.paused ||
        state.phase == ExecutionPhase.stepMode) {
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
    if (state.phase == ExecutionPhase.idle ||
        state.phase == ExecutionPhase.compiling) {
      return;
    }

    state = state.copyWith(phase: ExecutionPhase.stepMode, isPlaying: false);
    _streamSubscription?.cancel();

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
        await _fetchMemoryState();
      }
    } catch (e) {
      state = state.copyWith(errorMessage: '单步异常: $e');
    }
  }

  // ========== Seek / 拖动进度条 ==========

  Future<void> seekTo(int targetStep) async {
    if (!state.canSeek) return;

    // 即时响应：目标已在缓存中
    if (targetStep >= 0 &&
        targetStep <= state.maxCollectedStep &&
        targetStep < state.frameCache.length) {
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
        await _fetchMemoryState();
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
    // Stream 模式下 Rust 端尽快执行并推送，前端播放速度由 UI 刷新控制
    // 如需调整，可在此处控制帧动画速度或添加节流逻辑
  }

  // ========== 重置 ==========

  void reset() {
    _streamSubscription?.cancel();
    rust.resetSession();
    state = const UnifiedState();
  }

  /// 获取当前 step 的变量快照，WatchTab 等组件统一从这里读取。
  /// 若 frameCache 为空（非统一模式），返回空列表。
  List<types.ApiVariableSnapshot> get currentVariables {
    final frameCache = state.frameCache;
    final currentStep = state.currentStep;
    if (frameCache.isEmpty ||
        currentStep < 0 ||
        currentStep >= frameCache.length) {
      return [];
    }
    return frameCache[currentStep].localVars;
  }

  void onCodeChanged() {
    if (state.phase != ExecutionPhase.idle) {
      _streamSubscription?.cancel();
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
