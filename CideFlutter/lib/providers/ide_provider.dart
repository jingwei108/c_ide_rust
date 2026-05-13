import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import '../models/knowledge_card.dart';
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
  });

  static const _defaultBottomSlots = ['output', 'diagnostics', 'algorithm'];
  static const _defaultFloatingSlots = [
    'knowledge', 'pointer', 'arrayVis', 'memory', 'variables', 'watch', 'callstack',
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
  IdeState build() => const IdeState();

  void updateSource(String value) {
    state = state.copyWith(source: value);
  }

  Future<void> compile() async {
    state = state.copyWith(isCompiling: true, output: '', clearError: true);
    try {
      final result = await rust.compile(source: state.source);
      final diags = result.diagnostics;
      state = state.copyWith(
        isCompiling: false,
        diagnostics: diags,
        knowledgeCards: KnowledgeCard.findByErrorCodes(diags.map((d) => d.errorCode).toList()),
        algorithmMatches: result.algorithmMatches,
        isRunning: false,
        isStepMode: false,
        currentLine: 0,
        output: result.success ? '编译成功' : '编译失败',
      );
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
}
