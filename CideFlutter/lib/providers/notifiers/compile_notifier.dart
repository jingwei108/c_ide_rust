part of '../ide_notifier.dart';

/// 编译与编译后处理：编译、诊断、意图推断、学习进度 streak。
mixin CompileNotifierMixin on AutoDisposeNotifier<IdeState>, ProgressMixin {
  RustApiService get _rustApi => ref.read(rustApiServiceProvider);
  /// 仅执行编译并更新状态，不启动统一模式或运行代码。
  /// run()/step() 等需要编译后执行的场景应调用此方法，避免重复执行。
  Future<bool> compileOnly() async {
    state = state.copyWith(isCompiling: true, output: '', clearError: true);
    try {
      // 同步当前编辑器内容到 files
      final syncFiles =
          state.files.map((f) {
            if (f.filename == state.currentFile) {
              return f.copyWith(source: state.source);
            }
            return f;
          }).toList();
      state = state.copyWith(files: syncFiles);

      final result = await _rustApi.compileMulti(
        files:
            syncFiles
                .map((f) => CodeFile(filename: f.filename, source: f.source))
                .toList(),
      );
      final diags = result.diagnostics;

      // 更新学习进度
      _updateStreak();
      final progress = state.learningProgress;
      final newErrorsByCode = Map<String, int>.from(progress.errorsByCode);
      for (final d in diags.where((d) => d.severity == 0)) {
        final key = d.errorCode.toString();
        newErrorsByCode[key] = (newErrorsByCode[key] ?? 0) + 1;
      }
      // 追加编译记录（用于误解模式检测）
      final errorCodes =
          diags.where((d) => d.severity == 0).map((d) => d.errorCode).toList();
      final newRecord = CompileSnapshot(
        timestampMs: DateTime.now().millisecondsSinceEpoch,
        success: result.success,
        errorCodes: errorCodes,
      );
      final newRecords = [...progress.recentCompileRecords, newRecord];
      final trimmedRecords =
          newRecords.length > 20
              ? newRecords.sublist(newRecords.length - 20)
              : newRecords;

      final newProgress = progress.copyWith(
        totalCompiles: progress.totalCompiles + 1,
        successfulCompiles:
            progress.successfulCompiles + (result.success ? 1 : 0),
        failedCompiles: progress.failedCompiles + (result.success ? 0 : 1),
        errorsByCode: newErrorsByCode,
        recentCompileRecords: trimmedRecords,
      );
      // P3: infer code intent
      List<IntentScore> intentScores = [];
      if (result.success) {
        try {
          intentScores = await _rustApi.inferIntentFromSource(source: state.source);
        } catch (_) {
          // ignore intent inference errors
        }
      }

      state = state.copyWith(
        isCompiling: false,
        diagnostics: diags,
        knowledgeCards: KnowledgeCard.findByErrorCodes(
          diags.map((d) => d.errorCode).toList(),
        ),
        algorithmMatches: result.algorithmMatches,
        intentScores: intentScores,
        isRunning: false,
        isStepMode: false,
        currentLine: 0,
        output: result.success ? '编译成功' : '编译失败',
        learningProgress: newProgress,
      );
      await _saveProgress();
      return result.success;
    } catch (e) {
      state = state.copyWith(isCompiling: false, error: '编译异常: $e');
      return false;
    }
  }

  Future<void> compile() async {
    // 同步当前编辑器内容到 files
    final syncFiles =
        state.files.map((f) {
          if (f.filename == state.currentFile) {
            return f.copyWith(source: state.source);
          }
          return f;
        }).toList();

    final success = await compileOnly();
    // 编译成功后启动统一模式
    if (success) {
      final unifiedNotifier = ref.read(unifiedProvider.notifier);
      await unifiedNotifier.compileAndRunMulti(syncFiles);
    }
  }

  void _updateStreak() {
    final today = _todayString();
    final last = state.learningProgress.lastActiveDate;
    int streak = state.learningProgress.streakDays;
    if (last != today) {
      final yesterday = _yesterdayString();
      if (last == yesterday) {
        streak += 1;
      } else {
        streak = 1;
      }
    }
    state = state.copyWith(
      learningProgress: state.learningProgress.copyWith(
        lastActiveDate: today,
        streakDays: streak,
      ),
    );
  }

  String _todayString() {
    final now = DateTime.now();
    return '${now.year}-${now.month.toString().padLeft(2, '0')}-${now.day.toString().padLeft(2, '0')}';
  }

  String _yesterdayString() {
    final now = DateTime.now().subtract(const Duration(days: 1));
    return '${now.year}-${now.month.toString().padLeft(2, '0')}-${now.day.toString().padLeft(2, '0')}';
  }
}
