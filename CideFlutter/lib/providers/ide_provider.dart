import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import '../models/knowledge_card.dart';
import '../models/learning_progress.dart';
import '../services/learning_progress_service.dart';
// ignore: unused_import
import '../models/panel_item.dart';

final ideProvider = NotifierProvider<IdeNotifier, IdeState>(IdeNotifier.new);

class CodeTemplate {
  final String key;
  final String displayName;
  final String category;
  final String code;

  const CodeTemplate(this.key, this.displayName, this.category, this.code);

  static const List<CodeTemplate> defaults = [
    CodeTemplate('bubble', '冒泡排序', '排序',
      'void bubbleSort(int arr[], int n) {\n'
      '    for (int i = 0; i < n - 1; i++) {\n'
      '        for (int j = 0; j < n - i - 1; j++) {\n'
      '            if (arr[j] > arr[j + 1]) {\n'
      '                int temp = arr[j];\n'
      '                arr[j] = arr[j + 1];\n'
      '                arr[j + 1] = temp;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '}'),
    CodeTemplate('binary', '二分查找', '查找',
      'int binarySearch(int arr[], int n, int target) {\n'
      '    int left = 0, right = n - 1;\n'
      '    while (left <= right) {\n'
      '        int mid = left + (right - left) / 2;\n'
      '        if (arr[mid] == target) return mid;\n'
      '        if (arr[mid] < target) left = mid + 1;\n'
      '        else right = mid - 1;\n'
      '    }\n'
      '    return -1;\n'
      '}'),
    CodeTemplate('linked', '链表节点', '结构',
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n\n'
      'struct Node* createNode(int data) {\n'
      '    struct Node* newNode = (struct Node*)malloc(sizeof(struct Node));\n'
      '    newNode->data = data;\n'
      '    newNode->next = NULL;\n'
      '    return newNode;\n'
      '}'),
    CodeTemplate('quick', '快速排序', '排序',
      'void quickSort(int arr[], int low, int high) {\n'
      '    if (low < high) {\n'
      '        int pivot = partition(arr, low, high);\n'
      '        quickSort(arr, low, pivot - 1);\n'
      '        quickSort(arr, pivot + 1, high);\n'
      '    }\n'
      '}\n\n'
      'int partition(int arr[], int low, int high) {\n'
      '    int pivot = arr[high];\n'
      '    int i = low - 1;\n'
      '    for (int j = low; j < high; j++) {\n'
      '        if (arr[j] <= pivot) {\n'
      '            i++;\n'
      '            int temp = arr[i];\n'
      '            arr[i] = arr[j];\n'
      '            arr[j] = temp;\n'
      '        }\n'
      '    }\n'
      '    int temp = arr[i + 1];\n'
      '    arr[i + 1] = arr[high];\n'
      '    arr[high] = temp;\n'
      '    return i + 1;\n'
      '}'),
    CodeTemplate('factorial', '递归阶乘', '递归',
      'int factorial(int n) {\n'
      '    if (n <= 1) return 1;\n'
      '    return n * factorial(n - 1);\n'
      '}'),
    CodeTemplate('fib', '斐波那契', '递归',
      'int fibonacci(int n) {\n'
      '    if (n <= 1) return n;\n'
      '    return fibonacci(n - 1) + fibonacci(n - 2);\n'
      '}'),
    CodeTemplate('array', '数组遍历', '基础',
      'int arr[5] = {1, 2, 3, 4, 5};\n'
      'int sum = 0;\n'
      'for (int i = 0; i < 5; i++) {\n'
      '    sum = sum + arr[i];\n'
      '}\n'
      'printf("%d", sum);'),
    CodeTemplate('pointer', '指针交换', '指针',
      'void swap(int* a, int* b) {\n'
      '    int temp = *a;\n'
      '    *a = *b;\n'
      '    *b = temp;\n'
      '}'),
  ];
}

class IdeState {
  final String source;
  final bool isCompiling;
  final bool isRunning;
  final bool isStepMode;
  final String output;
  final List<rust.Diagnostic> diagnostics;
  final List<KnowledgeCard> knowledgeCards;
  final List<rust.AlgorithmMatch> algorithmMatches;
  final int currentLine;
  final int highlightedLine;
  final bool waitingInput;
  final String? error;
  final Set<int> breakpoints;

  // 面板布局状态
  final List<String> bottomSlots;
  final List<String> floatingSlots;
  final int bottomActiveIndex;
  final int floatingActiveIndex;
  final double bottomHeight;
  final bool isFloatingOpen;
  final List<String> watchExpressions;
  final int executionSpeed; // 0~500 ms
  final bool showIntro;
  final LearningProgress learningProgress;

  const IdeState({
    this.source = _defaultCode,
    this.isCompiling = false,
    this.isRunning = false,
    this.isStepMode = false,
    this.output = '',
    this.diagnostics = const [],
    this.knowledgeCards = const [],
    this.algorithmMatches = const [],
    this.currentLine = 0,
    this.highlightedLine = 0,
    this.waitingInput = false,
    this.error,
    this.breakpoints = const {},
    this.bottomSlots = _defaultBottomSlots,
    this.floatingSlots = _defaultFloatingSlots,
    this.bottomActiveIndex = 0,
    this.floatingActiveIndex = 0,
    this.bottomHeight = 220,
    this.isFloatingOpen = false,
    this.watchExpressions = const [],
    this.executionSpeed = 0,
    this.showIntro = false,
    this.learningProgress = const LearningProgress(),
  });

  static const _defaultBottomSlots = ['output', 'diagnostics', 'algorithm'];
  static const _defaultFloatingSlots = [
    'knowledge', 'pointer', 'arrayVis', 'memory', 'variables', 'watch', 'callstack', 'progress',
  ];

  IdeState copyWith({
    String? source,
    bool? isCompiling,
    bool? isRunning,
    bool? isStepMode,
    String? output,
    List<rust.Diagnostic>? diagnostics,
    List<KnowledgeCard>? knowledgeCards,
    List<rust.AlgorithmMatch>? algorithmMatches,
    int? currentLine,
    int? highlightedLine,
    bool? waitingInput,
    String? error,
    Set<int>? breakpoints,
    List<String>? bottomSlots,
    List<String>? floatingSlots,
    int? bottomActiveIndex,
    int? floatingActiveIndex,
    double? bottomHeight,
    bool? isFloatingOpen,
    List<String>? watchExpressions,
    int? executionSpeed,
    bool? showIntro,
    LearningProgress? learningProgress,
    bool clearError = false,
  }) {
    return IdeState(
      source: source ?? this.source,
      isCompiling: isCompiling ?? this.isCompiling,
      isRunning: isRunning ?? this.isRunning,
      isStepMode: isStepMode ?? this.isStepMode,
      output: output ?? this.output,
      diagnostics: diagnostics ?? this.diagnostics,
      knowledgeCards: knowledgeCards ?? this.knowledgeCards,
      algorithmMatches: algorithmMatches ?? this.algorithmMatches,
      currentLine: currentLine ?? this.currentLine,
      highlightedLine: highlightedLine ?? this.highlightedLine,
      waitingInput: waitingInput ?? this.waitingInput,
      error: clearError ? null : (error ?? this.error),
      breakpoints: breakpoints ?? this.breakpoints,
      bottomSlots: bottomSlots ?? this.bottomSlots,
      floatingSlots: floatingSlots ?? this.floatingSlots,
      bottomActiveIndex: bottomActiveIndex ?? this.bottomActiveIndex,
      floatingActiveIndex: floatingActiveIndex ?? this.floatingActiveIndex,
      bottomHeight: bottomHeight ?? this.bottomHeight,
      isFloatingOpen: isFloatingOpen ?? this.isFloatingOpen,
      watchExpressions: watchExpressions ?? this.watchExpressions,
      executionSpeed: executionSpeed ?? this.executionSpeed,
      showIntro: showIntro ?? this.showIntro,
      learningProgress: learningProgress ?? this.learningProgress,
    );
  }

  bool get hasErrors => diagnostics.any((d) => d.severity == 0);
  bool get hasWarnings => diagnostics.any((d) => d.severity == 1);

  static const _defaultCode = '''#include <stdio.h>

int main() {
    printf("Hello, Cide!\\n");
    return 0;
}
''';
}

class IdeNotifier extends Notifier<IdeState> {
  final _outputController = TextEditingController();
  TextEditingController get outputController => _outputController;

  @override
  IdeState build() {
    // 延迟加载持久化进度
    Future.microtask(_loadProgress);
    return const IdeState();
  }

  Future<void> _loadProgress() async {
    final progress = await LearningProgressService.load();
    state = state.copyWith(learningProgress: progress);
  }

  Future<void> _saveProgress() async {
    await LearningProgressService.save(state.learningProgress);
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

  void updateSource(String value) {
    state = state.copyWith(source: value);
  }

  Future<void> compile() async {
    state = state.copyWith(isCompiling: true, output: '', clearError: true);
    try {
      final result = await rust.compile(source: state.source);
      final diags = result.diagnostics;

      // 更新学习进度
      _updateStreak();
      final progress = state.learningProgress;
      final newErrorsByCode = Map<String, int>.from(progress.errorsByCode);
      for (final d in diags.where((d) => d.severity == 0)) {
        final key = d.errorCode.toString();
        newErrorsByCode[key] = (newErrorsByCode[key] ?? 0) + 1;
      }
      final newProgress = progress.copyWith(
        totalCompiles: progress.totalCompiles + 1,
        successfulCompiles: progress.successfulCompiles + (result.success ? 1 : 0),
        failedCompiles: progress.failedCompiles + (result.success ? 0 : 1),
        errorsByCode: newErrorsByCode,
      );
      state = state.copyWith(
        isCompiling: false,
        diagnostics: diags,
        knowledgeCards: KnowledgeCard.findByErrorCodes(diags.map((d) => d.errorCode).toList()),
        algorithmMatches: result.algorithmMatches,
        isRunning: false,
        isStepMode: false,
        currentLine: 0,
        output: result.success ? '编译成功' : '编译失败',
        learningProgress: newProgress,
      );
      await _saveProgress();
    } catch (e) {
      state = state.copyWith(isCompiling: false, error: '编译异常: $e');
    }
  }

  Future<void> run() async {
    if (state.hasErrors) {
      state = state.copyWith(error: '请先修复编译错误');
      return;
    }
    state = state.copyWith(isRunning: true, isStepMode: false, output: '', clearError: true);
    try {
      final result = await rust.runCode();
      state = state.copyWith(
        isRunning: result.waitingInput,
        output: result.output,
        waitingInput: result.waitingInput,
        error: result.error,
      );
    } catch (e) {
      state = state.copyWith(isRunning: false, error: '运行异常: $e');
    }
  }

  Future<void> step() async {
    if (state.hasErrors) {
      state = state.copyWith(error: '请先修复编译错误');
      return;
    }
    state = state.copyWith(isStepMode: true, isRunning: true, clearError: true);
    try {
      final result = await rust.stepNext();
      if (state.executionSpeed > 0) {
        await Future.delayed(Duration(milliseconds: state.executionSpeed));
      }
      state = state.copyWith(
        isRunning: result.status != rust.StepStatus.finished && result.status != rust.StepStatus.trap,
        isStepMode: result.status != rust.StepStatus.finished && result.status != rust.StepStatus.trap,
        currentLine: result.currentLine,
        output: result.output,
        waitingInput: result.waitingInput,
        error: result.status == rust.StepStatus.trap ? '运行出错' : null,
      );
    } catch (e) {
      state = state.copyWith(isRunning: false, isStepMode: false, error: '单步异常: $e');
    }
  }

  Future<void> provideInput(String line) async {
    try {
      await rust.provideInputLine(line: line);
      if (state.isStepMode) {
        await step();
      } else {
        await run();
      }
    } catch (e) {
      state = state.copyWith(error: '输入异常: $e');
    }
  }

  void reset() {
    rust.resetSession();
    state = const IdeState();
  }

  void clearOutput() {
    state = state.copyWith(output: '');
  }

  void clearError() {
    state = state.copyWith(clearError: true);
  }

  void toggleBreakpoint(int line) {
    final newBreakpoints = Set<int>.from(state.breakpoints);
    if (newBreakpoints.contains(line)) {
      newBreakpoints.remove(line);
      rust.clearBreakpoints();
      for (final bp in newBreakpoints) {
        rust.addBreakpoint(line: bp);
      }
    } else {
      newBreakpoints.add(line);
      rust.addBreakpoint(line: line);
    }
    state = state.copyWith(breakpoints: newBreakpoints);
  }

  // ========== 面板管理 ==========

  /// 选择底部 Tab
  void selectBottomTab(int index) {
    state = state.copyWith(bottomActiveIndex: index);
  }

  /// 选择悬浮球 Tab
  void selectFloatingTab(int index) {
    state = state.copyWith(floatingActiveIndex: index);
  }

  /// 设置底部面板高度
  void setBottomHeight(double height) {
    state = state.copyWith(bottomHeight: height.clamp(120, 500));
  }

  /// 切换悬浮球展开/收起
  void toggleFloating() {
    state = state.copyWith(isFloatingOpen: !state.isFloatingOpen);
  }

  /// 关闭悬浮球
  void closeFloating() {
    state = state.copyWith(isFloatingOpen: false);
  }

  /// 将面板从当前位置移动到底部
  void moveToBottom(String panelId) {
    final currentFloating = List<String>.from(state.floatingSlots);
    final currentBottom = List<String>.from(state.bottomSlots);

    if (currentBottom.contains(panelId)) return;

    // 底部最多放 3 个
    if (currentBottom.length >= 3) {
      // 底部已满，把底部的最后一个移到悬浮球
      final overflow = currentBottom.removeLast();
      if (!currentFloating.contains(overflow)) {
        currentFloating.add(overflow);
      }
    }

    currentFloating.remove(panelId);
    currentBottom.add(panelId);

    state = state.copyWith(
      bottomSlots: currentBottom,
      floatingSlots: currentFloating,
      bottomActiveIndex: currentBottom.indexOf(panelId).clamp(0, currentBottom.length - 1),
    );
  }

  /// 将面板从当前位置移动到悬浮球
  void moveToFloating(String panelId) {
    final currentFloating = List<String>.from(state.floatingSlots);
    final currentBottom = List<String>.from(state.bottomSlots);

    if (currentFloating.contains(panelId)) return;

    // 悬浮球最多放 7 个
    if (currentFloating.length >= 7) {
      state = state.copyWith(error: '悬浮球承载已达上限（最多7个）');
      return;
    }

    currentBottom.remove(panelId);
    currentFloating.add(panelId);

    state = state.copyWith(
      bottomSlots: currentBottom,
      floatingSlots: currentFloating,
      floatingActiveIndex: currentFloating.indexOf(panelId).clamp(0, currentFloating.length - 1),
    );
  }

  /// 交换底部两个面板位置
  void swapBottomPanels(int indexA, int indexB) {
    final slots = List<String>.from(state.bottomSlots);
    if (indexA < 0 || indexA >= slots.length) return;
    if (indexB < 0 || indexB >= slots.length) return;
    final temp = slots[indexA];
    slots[indexA] = slots[indexB];
    slots[indexB] = temp;
    state = state.copyWith(bottomSlots: slots);
  }

  /// 交换悬浮球两个面板位置
  void swapFloatingPanels(int indexA, int indexB) {
    final slots = List<String>.from(state.floatingSlots);
    if (indexA < 0 || indexA >= slots.length) return;
    if (indexB < 0 || indexB >= slots.length) return;
    final temp = slots[indexA];
    slots[indexA] = slots[indexB];
    slots[indexB] = temp;
    state = state.copyWith(floatingSlots: slots);
  }

  /// 双击底部面板标题：删除并移到悬浮球
  void removeBottomPanel(int index) {
    final bottom = List<String>.from(state.bottomSlots);
    final floating = List<String>.from(state.floatingSlots);
    if (index < 0 || index >= bottom.length) return;

    final panelId = bottom.removeAt(index);
    if (!floating.contains(panelId)) {
      if (floating.length >= 7) {
        state = state.copyWith(error: '悬浮球承载已达上限（最多7个）');
        return;
      }
      floating.add(panelId);
    }

    state = state.copyWith(
      bottomSlots: bottom,
      floatingSlots: floating,
      bottomActiveIndex: state.bottomActiveIndex.clamp(0, (bottom.length - 1).clamp(0, 999)),
    );
  }

  /// 双击悬浮球面板标题：删除并移到底部
  void removeFloatingPanel(int index) {
    final bottom = List<String>.from(state.bottomSlots);
    final floating = List<String>.from(state.floatingSlots);
    if (index < 0 || index >= floating.length) return;

    final panelId = floating.removeAt(index);
    if (!bottom.contains(panelId)) {
      if (bottom.length >= 3) {
        // 底部已满，把最后一个移到悬浮球
        final overflow = bottom.removeLast();
        if (!floating.contains(overflow)) {
          floating.add(overflow);
        }
      }
      bottom.add(panelId);
    }

    state = state.copyWith(
      bottomSlots: bottom,
      floatingSlots: floating,
      floatingActiveIndex: state.floatingActiveIndex.clamp(0, (floating.length - 1).clamp(0, 999)),
    );
  }

  void highlightLine(int line) {
    state = state.copyWith(highlightedLine: line);
  }

  void clearHighlight() {
    state = state.copyWith(highlightedLine: 0);
  }

  // ========== 应用修复 ==========

  Future<String?> applyFix(rust.Diagnostic diag) async {
    final source = state.source;
    // 1. 尝试结构化替换
    if ((diag.fixKind == 1 || diag.fixKind == 2) && diag.replaceStartLine > 0) {
      final lines = source.replaceAll('\r\n', '\n').split('\n');
      final startLine = diag.replaceStartLine - 1;
      final endLine = diag.replaceEndLine - 1;
      if (startLine >= 0 && startLine < lines.length && endLine >= 0 && endLine < lines.length) {
        if (startLine == endLine) {
          final line = lines[startLine];
          final startCol = diag.replaceStartColumn;
          final endCol = diag.replaceEndColumn;
          if (startCol >= 0 && endCol <= line.length && startCol <= endCol) {
            final before = line.substring(0, startCol);
            final after = line.substring(endCol);
            lines[startLine] = before + diag.replacementText + after;
            final newSource = lines.join('\n');
            state = state.copyWith(source: newSource);
            await _recordFix(diag.errorCode);
            return '已应用修复（第${diag.line}行）';
          }
        }
      }
    }

    // 2. 回退到启发式字符串匹配
    final lines = source.replaceAll('\r\n', '\n').split('\n');
    final lineIndex = diag.line - 1;
    if (lineIndex < 0 || lineIndex >= lines.length) return null;

    final fix = diag.fixSuggestion;
    bool applied = false;

    if (fix.contains('分号') || fix.contains("';'")) {
      final trimmed = lines[lineIndex].trimRight();
      if (!trimmed.endsWith(';') && !trimmed.endsWith('{') && !trimmed.endsWith('}')) {
        lines[lineIndex] = '$trimmed;';
        applied = true;
      }
    } else if (fix.contains('右花括号') || fix.contains("'}'")) {
      final trimmed = lines[lineIndex].trimRight();
      if (!trimmed.endsWith('}')) {
        lines[lineIndex] = '$trimmed}';
        applied = true;
      }
    } else if (fix.contains('右圆括号') || fix.contains("')'")) {
      final trimmed = lines[lineIndex].trimRight();
      if (!trimmed.endsWith(')')) {
        lines[lineIndex] = '$trimmed)';
        applied = true;
      }
    } else if (fix.contains('右方括号') || fix.contains("']'")) {
      final trimmed = lines[lineIndex].trimRight();
      if (!trimmed.endsWith(']')) {
        lines[lineIndex] = '$trimmed]';
        applied = true;
      }
    } else if (fix.contains('双引号') || fix.contains('"""')) {
      final trimmed = lines[lineIndex].trimRight();
      if (!trimmed.endsWith('"')) {
        lines[lineIndex] = '$trimmed"';
        applied = true;
      }
    } else if (fix.contains("=' 改为 '=='") || fix.contains('==')) {
      final line = lines[lineIndex];
      final parenStart = line.indexOf('(');
      final parenEnd = line.lastIndexOf(')');
      if (parenStart >= 0 && parenEnd > parenStart) {
        final before = line.substring(0, parenStart + 1);
        final cond = line.substring(parenStart + 1, parenEnd);
        final after = line.substring(parenEnd);
        final sb = StringBuffer();
        for (var i = 0; i < cond.length; i++) {
          if (cond[i] == '=') {
            final precededByOp = i > 0 && (cond[i - 1] == '=' || cond[i - 1] == '!' || cond[i - 1] == '<' || cond[i - 1] == '>');
            final followedByEq = i + 1 < cond.length && cond[i + 1] == '=';
            if (!precededByOp && !followedByEq) {
              sb.write('==');
              applied = true;
              continue;
            }
          }
          sb.write(cond[i]);
        }
        if (applied) {
          lines[lineIndex] = before + sb.toString() + after;
        }
      }
    } else if (fix.contains("'<=' 改为 '<'") || fix.contains('<')) {
      final trimmed = lines[lineIndex];
      final idx = trimmed.indexOf('<=');
      if (idx >= 0) {
        lines[lineIndex] = '${trimmed.substring(0, idx)}<${trimmed.substring(idx + 2)}';
        applied = true;
      }
    } else if (fix.contains('-> 改为 .')) {
      final trimmed = lines[lineIndex];
      final idx = trimmed.indexOf('->');
      if (idx >= 0) {
        lines[lineIndex] = '${trimmed.substring(0, idx)}.${trimmed.substring(idx + 2)}';
        applied = true;
      }
    } else if (fix.contains('return 0;')) {
      final trimmed = lines[lineIndex].trimRight();
      if (!trimmed.endsWith(';')) {
        lines[lineIndex] = '$trimmed;';
        applied = true;
      } else if (!trimmed.contains('return')) {
        lines[lineIndex] = '$trimmed return 0;';
        applied = true;
      }
    }

    if (applied) {
      final newSource = lines.join('\n');
      state = state.copyWith(source: newSource);
      await _recordFix(diag.errorCode);
      return '已应用修复（第${diag.line}行）：$fix';
    }

    return null;
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
    final newViewed = Set<String>.from(progress.viewedKnowledgeCards)..add(cardId);
    state = state.copyWith(
      learningProgress: progress.copyWith(viewedKnowledgeCards: newViewed),
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

  // ========== 算法验证 ==========

  Future<AlgorithmValidationResult> validateAlgorithm(rust.AlgorithmMatch match) async {
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
    final newPassed = Map<String, int>.from(progress.algorithmValidationsPassed);
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
    return AlgorithmValidationResult(true, '✅ ${match.displayName} 通过了 ${testCases.length} 组测试用例！');
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
      final compileResult = await rust.compile(source: harness);
      if (!compileResult.success) {
        return AlgorithmValidationResult(false, '测试用例「${tc.description}」编译失败');
      }
      final runResult = await rust.runCode();
      if (!runResult.success || runResult.error != null) {
        return AlgorithmValidationResult(false, '测试用例「${tc.description}」运行时错误: ${runResult.error}');
      }
      return _verifyOutput(algorithmName, tc, runResult.output.trim());
    } catch (e) {
      return AlgorithmValidationResult(false, '测试用例「${tc.description}」异常: $e');
    }
  }

  String _buildHarness(String sourceCode, String funcName, String algorithmName, AlgorithmTestCase tc) {
    // 替换学生的 main() 以便注入我们自己的 main
    final modifiedSource = sourceCode.replaceAll(RegExp(r'(?<!\w)int\s+main\s*\('), 'int __cide_original_main(');
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
    if (['bubble_sort', 'selection_sort', 'insertion_sort', 'quick_sort', 'merge_sort'].contains(algorithmName)) {
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

  AlgorithmValidationResult _verifyOutput(String algorithmName, AlgorithmTestCase tc, String output) {
    if (['bubble_sort', 'selection_sort', 'insertion_sort', 'quick_sort', 'merge_sort'].contains(algorithmName)) {
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
      return AlgorithmValidationResult(false,
        '输出长度不匹配。期望 ${tc.inputArray.length} 个元素，实际得到 ${actual.length} 个。');
    }
    for (var i = 1; i < actual.length; i++) {
      if (actual[i] < actual[i - 1]) {
        return AlgorithmValidationResult(false,
          '排序结果不是非递减的。arr[${i - 1}] = ${actual[i - 1]}，arr[$i] = ${actual[i]}。');
      }
    }
    final expectedSorted = List<int>.from(tc.inputArray)..sort();
    for (var i = 0; i < actual.length; i++) {
      if (actual[i] != expectedSorted[i]) {
        return AlgorithmValidationResult(false,
          '元素守恒被破坏。排序后 arr[$i] = ${actual[i]}，但期望 ${expectedSorted[i]}。');
      }
    }
    return AlgorithmValidationResult(true, '');
  }

  AlgorithmValidationResult _verifyBinarySearch(AlgorithmTestCase tc, String output) {
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
        return AlgorithmValidationResult(false,
          '目标 ${tc.searchTarget} 不在数组中，应返回 -1，但返回了 $actualIndex。');
      } else {
        return AlgorithmValidationResult(false,
          '目标 ${tc.searchTarget} 应在索引 $expectedIndex 处，但返回了 $actualIndex。');
      }
    }
    return AlgorithmValidationResult(true, '');
  }
}

class AlgorithmTestCase {
  final String description;
  final List<int> inputArray;
  final int? searchTarget;
  AlgorithmTestCase(this.description, this.inputArray, [this.searchTarget]);
}

class AlgorithmValidationResult {
  final bool passed;
  final String message;
  AlgorithmValidationResult(this.passed, this.message);
}
