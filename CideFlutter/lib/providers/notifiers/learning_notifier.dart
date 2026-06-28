part of '../ide_notifier.dart';

/// 学习相关功能：进度持久化、修复记录、教程、算法验证、Watch/Intro。
mixin LearningNotifierMixin
    on
        AutoDisposeNotifier<IdeState>,
        ProgressMixin,
        FileNotifierMixin,
        CompileNotifierMixin {
  RustApiService get _rustApi => ref.read(rustApiServiceProvider);
  // ========== 应用修复 ==========

  Future<String?> applyFix(rust.Diagnostic diag) async {
    final targetFilename = diag.filename;
    final file = state.files.firstWhere(
      (f) => f.filename == targetFilename,
      orElse:
          () => state.files.firstWhere((f) => f.filename == state.currentFile),
    );
    final source = file.source;

    final newSource = await _rustApi.applyFix(source: source, diag: diag);
    if (newSource == null) return null;

    setFileSource(targetFilename, newSource);
    await _recordFix(diag.errorCode);
    return '已应用修复（${diag.filename}:${diag.line}）';
  }

  Future<void> _recordFix(int errorCode) async {
    final progress = state.learningProgress;
    final newFixed = Map<String, int>.from(progress.fixedErrorsByCode);
    final key = errorCode.toString();
    newFixed[key] = (newFixed[key] ?? 0) + 1;
    state = state.copyWith(
      learningProgress: progress.copyWith(fixedErrorsByCode: newFixed),
    );
    await _saveProgress();
  }

  /// 记录用户查看了一张知识卡片
  Future<void> recordKnowledgeCardView(String cardId) async {
    final progress = state.learningProgress;
    if (progress.viewedKnowledgeCards.contains(cardId)) return;
    final newViewed = Set<String>.from(progress.viewedKnowledgeCards)
      ..add(cardId);
    state = state.copyWith(
      learningProgress: progress.copyWith(viewedKnowledgeCards: newViewed),
    );
    await _saveProgress();
  }

  /// 记录一次统一模式运行
  Future<void> recordUnifiedRun({
    required int steps,
    required bool trapped,
    String? trapMessage,
  }) async {
    final progress = state.learningProgress;
    var records = progress.recentCompileRecords;
    // 如果有 trap，更新最近一条编译记录的 trapMessage
    if (trapped && trapMessage != null && records.isNotEmpty) {
      final last = records.last;
      records = [...records];
      records[records.length - 1] = CompileSnapshot(
        timestampMs: last.timestampMs,
        success: last.success,
        errorCodes: last.errorCodes,
        trapMessage: trapMessage,
      );
    }
    state = state.copyWith(
      learningProgress: progress.copyWith(
        totalUnifiedRuns: progress.totalUnifiedRuns + 1,
        totalStepsExecuted: progress.totalStepsExecuted + steps,
        maxStepsInSingleRun: math.max(progress.maxStepsInSingleRun, steps),
        totalTraps: trapped ? progress.totalTraps + 1 : progress.totalTraps,
        recentCompileRecords: records,
      ),
    );
    await _saveProgress();
  }

  /// 记录一次 Seek 操作
  Future<void> recordSeek() async {
    final progress = state.learningProgress;
    state = state.copyWith(
      learningProgress: progress.copyWith(totalSeeks: progress.totalSeeks + 1),
    );
    await _saveProgress();
  }

  /// 重置学习进度
  Future<void> resetProgress() async {
    await LearningProgressService.clear();
    state = state.copyWith(learningProgress: const LearningProgress());
  }

  // ========== 监视变量 ==========

  void addWatchExpression(String expr) {
    expr = expr.trim();
    if (expr.isEmpty) return;
    if (state.watchExpressions.contains(expr)) return;
    state = state.copyWith(watchExpressions: [...state.watchExpressions, expr]);
  }

  void removeWatchExpression(String expr) {
    state = state.copyWith(
      watchExpressions: state.watchExpressions.where((e) => e != expr).toList(),
    );
  }

  void clearWatchExpressions() {
    state = state.copyWith(watchExpressions: const []);
  }

  void setExecutionSpeed(int speed) {
    state = state.copyWith(executionSpeed: speed.clamp(0, 500));
  }

  void showIntro() {
    state = state.copyWith(showIntro: true);
  }

  void hideIntro() {
    state = state.copyWith(showIntro: false);
  }

  // ========== 模板教程 ==========

  void startTutorial(CodeTemplate template, String generatedCode) {
    final steps = template.tutorialSteps;
    if (steps.isEmpty) return;
    final focusLines = steps.first.focusLines;
    state = state.copyWith(
      activeTutorial: TutorialSession(
        templateKey: template.key,
        templateExt: template.ext,
        generatedCode: generatedCode,
        stepIndex: 0,
        focusLines: focusLines,
        steps: steps,
      ),
    );
  }

  void nextTutorialStep() {
    final tutorial = state.activeTutorial;
    if (tutorial == null) return;
    final nextIndex = tutorial.stepIndex + 1;
    if (nextIndex >= tutorial.steps.length) {
      completeTutorial();
      return;
    }
    final nextStep = tutorial.steps[nextIndex];
    state = state.copyWith(
      activeTutorial: tutorial.copyWith(
        stepIndex: nextIndex,
        focusLines: nextStep.focusLines,
      ),
    );
  }

  void prevTutorialStep() {
    final tutorial = state.activeTutorial;
    if (tutorial == null) return;
    final prevIndex = tutorial.stepIndex - 1;
    if (prevIndex < 0) return;
    final prevStep = tutorial.steps[prevIndex];
    state = state.copyWith(
      activeTutorial: tutorial.copyWith(
        stepIndex: prevIndex,
        focusLines: prevStep.focusLines,
      ),
    );
  }

  void skipTutorial() {
    completeTutorial();
  }

  Future<void> completeTutorial() async {
    final tutorial = state.activeTutorial;
    if (tutorial == null) return;

    // 记录已完成
    final progress = state.learningProgress;
    final newCompleted = Set<String>.from(progress.completedTutorials)
      ..add(tutorial.templateKey);

    // 根据模板语言确定目标文件名，确保 Rust 后端按 C/C++ 模式编译。
    final isCpp = tutorial.templateExt == 'cpp';
    final targetFilename = isCpp ? 'main.cpp' : 'main.c';
    final generatedCode = tutorial.generatedCode;

    // 如果目标文件已存在且不是当前文件，直接切换过去并替换源码；
    // 否则替换当前文件内容，同时把文件名改成目标扩展名。
    final targetExists = state.files.any((f) => f.filename == targetFilename);
    final newFiles = state.files.map((f) {
      if (f.filename == state.currentFile && !targetExists) {
        return CodeFile(filename: targetFilename, source: generatedCode);
      }
      if (f.filename == targetFilename) {
        return f.copyWith(source: generatedCode);
      }
      return f;
    }).toList();

    state = state.copyWith(
      clearActiveTutorial: true,
      source: generatedCode,
      currentFile: targetFilename,
      files: newFiles,
      learningProgress: progress.copyWith(completedTutorials: newCompleted),
    );

    await _saveProgress();

    // 自动编译运行（会启动统一模式）
    await compile();
  }

  // ========== 算法验证 ==========

  Future<AlgorithmValidationResult> validateAlgorithm(
    rust.AlgorithmMatch match,
  ) async {
    if (match.funcName.isEmpty) {
      return AlgorithmValidationResult(false, '无法获取函数名，无法验证算法。');
    }
    final testCases = _generateTestCases(match.name);
    if (testCases.isEmpty) {
      return AlgorithmValidationResult(false, '暂不支持验证算法: ${match.displayName}');
    }
    bool allPassed = true;
    String? failMessage;
    for (final tc in testCases) {
      final result = await _runSingleTest(match.funcName, match.name, tc);
      if (!result.passed) {
        allPassed = false;
        failMessage = result.message;
        break;
      }
    }
    // 记录验证结果
    final progress = state.learningProgress;
    final newPassed = Map<String, int>.from(
      progress.algorithmValidationsPassed,
    );
    final newTotal = Map<String, int>.from(progress.algorithmValidationsTotal);
    newTotal[match.name] = (newTotal[match.name] ?? 0) + 1;
    if (allPassed) {
      newPassed[match.name] = (newPassed[match.name] ?? 0) + 1;
    }
    state = state.copyWith(
      learningProgress: progress.copyWith(
        algorithmValidationsPassed: newPassed,
        algorithmValidationsTotal: newTotal,
      ),
    );
    await _saveProgress();

    if (!allPassed) {
      return AlgorithmValidationResult(false, failMessage ?? '验证失败');
    }
    return AlgorithmValidationResult(
      true,
      '✅ ${match.displayName} 通过了 ${testCases.length} 组测试用例！',
    );
  }

  List<AlgorithmTestCase> _generateTestCases(String algorithmName) {
    switch (algorithmName) {
      case 'bubble_sort':
      case 'selection_sort':
      case 'insertion_sort':
      case 'quick_sort':
      case 'merge_sort':
        return [
          AlgorithmTestCase('随机数组', [5, 3, 8, 1, 2]),
          AlgorithmTestCase('已有序', [1, 2, 3, 4, 5]),
          AlgorithmTestCase('逆序', [5, 4, 3, 2, 1]),
          AlgorithmTestCase('单元素', [42]),
          AlgorithmTestCase('全部相同', [2, 2, 2, 2]),
          AlgorithmTestCase('空数组', []),
          AlgorithmTestCase('包含负数', [-3, 5, -1, 0, 2]),
        ];
      case 'binary_search':
        return [
          AlgorithmTestCase('找到目标', [1, 3, 5, 7, 9], 5),
          AlgorithmTestCase('找到首个', [1, 3, 5, 7, 9], 1),
          AlgorithmTestCase('找到末尾', [1, 3, 5, 7, 9], 9),
          AlgorithmTestCase('未找到（偏小）', [1, 3, 5, 7, 9], 0),
          AlgorithmTestCase('未找到（偏大）', [1, 3, 5, 7, 9], 10),
          AlgorithmTestCase('单元素找到', [5], 5),
          AlgorithmTestCase('单元素未找到', [5], 3),
          AlgorithmTestCase('空数组', [], 1),
        ];
      default:
        return [];
    }
  }

  Future<AlgorithmValidationResult> _runSingleTest(
    String funcName,
    String algorithmName,
    AlgorithmTestCase tc,
  ) async {
    final harness = _buildHarness(state.source, funcName, algorithmName, tc);
    if (harness.isEmpty) {
      return AlgorithmValidationResult(false, '生成测试代码失败。');
    }
    try {
      final compileResult = await _rustApi.compile(source: harness);
      if (!compileResult.success) {
        return AlgorithmValidationResult(false, '测试用例「${tc.description}」编译失败');
      }
      final runResult = await _rustApi.runCode();
      if (!runResult.success || runResult.error != null) {
        return AlgorithmValidationResult(
          false,
          '测试用例「${tc.description}」运行时错误: ${runResult.error}',
        );
      }
      return _verifyOutput(algorithmName, tc, runResult.output.trim());
    } catch (e) {
      return AlgorithmValidationResult(false, '测试用例「${tc.description}」异常: $e');
    }
  }

  String _replaceMainSafely(String code) {
    // Build a mask marking characters inside comments or string literals
    final mask = List<bool>.filled(code.length, false);
    int i = 0;
    while (i < code.length) {
      if (i + 1 < code.length && code[i] == '/' && code[i + 1] == '/') {
        while (i < code.length && code[i] != '\n') {
          mask[i] = true;
          i++;
        }
      } else if (i + 1 < code.length && code[i] == '/' && code[i + 1] == '*') {
        mask[i] = true;
        i++;
        mask[i] = true;
        i++;
        while (i + 1 < code.length && !(code[i] == '*' && code[i + 1] == '/')) {
          mask[i] = true;
          i++;
        }
        if (i < code.length) {
          mask[i] = true;
          i++;
        }
        if (i < code.length) {
          mask[i] = true;
          i++;
        }
      } else if (code[i] == '"') {
        mask[i] = true;
        i++;
        while (i < code.length && code[i] != '"') {
          if (code[i] == '\\' && i + 1 < code.length) {
            mask[i] = true;
            i++;
            mask[i] = true;
            i++;
          } else {
            mask[i] = true;
            i++;
          }
        }
        if (i < code.length) {
          mask[i] = true;
          i++;
        }
      } else {
        i++;
      }
    }

    final pattern = RegExp(r'(?<!\w)int\s+main\s*\(');
    return code.replaceAllMapped(pattern, (match) {
      final start = match.start;
      if (start >= 0 && start < mask.length && !mask[start]) {
        return 'int __cide_original_main(';
      }
      return match.group(0)!;
    });
  }

  String _buildHarness(
    String sourceCode,
    String funcName,
    String algorithmName,
    AlgorithmTestCase tc,
  ) {
    // 替换学生的 main() 以便注入我们自己的 main，但跳过注释和字符串中的匹配
    final modifiedSource = _replaceMainSafely(sourceCode);
    final sb = StringBuffer();
    sb.writeln(modifiedSource);
    sb.writeln();
    sb.writeln('int main() {');
    if (tc.inputArray.isEmpty) {
      sb.writeln('    int* arr = 0;');
      sb.writeln('    int n = 0;');
    } else {
      sb.write('    int arr[] = {');
      for (var i = 0; i < tc.inputArray.length; i++) {
        if (i > 0) sb.write(', ');
        sb.write(tc.inputArray[i]);
      }
      sb.writeln('};');
      sb.writeln('    int n = ${tc.inputArray.length};');
    }
    if ([
      'bubble_sort',
      'selection_sort',
      'insertion_sort',
      'quick_sort',
      'merge_sort',
    ].contains(algorithmName)) {
      sb.writeln('    $funcName(arr, n);');
      sb.writeln('    for (int i = 0; i < n; i = i + 1) {');
      sb.writeln('        printf("%d ", arr[i]);');
      sb.writeln('    }');
    } else if (algorithmName == 'binary_search') {
      if (tc.searchTarget != null) {
        sb.writeln('    int result = $funcName(arr, n, ${tc.searchTarget});');
        sb.writeln('    printf("%d", result);');
      } else {
        return '';
      }
    } else {
      return '';
    }
    sb.writeln('    return 0;');
    sb.writeln('}');
    return sb.toString();
  }

  AlgorithmValidationResult _verifyOutput(
    String algorithmName,
    AlgorithmTestCase tc,
    String output,
  ) {
    if ([
      'bubble_sort',
      'selection_sort',
      'insertion_sort',
      'quick_sort',
      'merge_sort',
    ].contains(algorithmName)) {
      return _verifySorted(tc, output);
    }
    if (algorithmName == 'binary_search') {
      return _verifyBinarySearch(tc, output);
    }
    return AlgorithmValidationResult(false, '未知算法类型: $algorithmName');
  }

  AlgorithmValidationResult _verifySorted(AlgorithmTestCase tc, String output) {
    final parts = output.split(' ').where((p) => p.trim().isNotEmpty).toList();
    final actual = <int>[];
    for (final p in parts) {
      final v = int.tryParse(p.trim());
      if (v != null) actual.add(v);
    }
    if (actual.length != tc.inputArray.length) {
      return AlgorithmValidationResult(
        false,
        '输出长度不匹配。期望 ${tc.inputArray.length} 个元素，实际得到 ${actual.length} 个。',
      );
    }
    for (var i = 1; i < actual.length; i++) {
      if (actual[i] < actual[i - 1]) {
        return AlgorithmValidationResult(
          false,
          '排序结果不是非递减的。arr[${i - 1}] = ${actual[i - 1]}，arr[$i] = ${actual[i]}。',
        );
      }
    }
    final expectedSorted = List<int>.from(tc.inputArray)..sort();
    for (var i = 0; i < actual.length; i++) {
      if (actual[i] != expectedSorted[i]) {
        return AlgorithmValidationResult(
          false,
          '元素守恒被破坏。排序后 arr[$i] = ${actual[i]}，但期望 ${expectedSorted[i]}。',
        );
      }
    }
    return AlgorithmValidationResult(true, '');
  }

  AlgorithmValidationResult _verifyBinarySearch(
    AlgorithmTestCase tc,
    String output,
  ) {
    final actualIndex = int.tryParse(output.trim());
    if (actualIndex == null) {
      return AlgorithmValidationResult(false, '输出无法解析为整数: "$output"');
    }
    final sorted = List<int>.from(tc.inputArray)..sort();
    var expectedIndex = -1;
    for (var i = 0; i < sorted.length; i++) {
      if (sorted[i] == tc.searchTarget) {
        expectedIndex = i;
        break;
      }
    }
    if (actualIndex != expectedIndex) {
      if (expectedIndex == -1) {
        return AlgorithmValidationResult(
          false,
          '目标 ${tc.searchTarget} 不在数组中，应返回 -1，但返回了 $actualIndex。',
        );
      } else {
        return AlgorithmValidationResult(
          false,
          '目标 ${tc.searchTarget} 应在索引 $expectedIndex 处，但返回了 $actualIndex。',
        );
      }
    }
    return AlgorithmValidationResult(true, '');
  }
}
