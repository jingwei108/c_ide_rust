import 'package:cide/src/rust/api/types.dart' as rust;
import 'package:cide/src/rust/compiler/intent.dart';
import 'code_template.dart';
import 'knowledge_card.dart';
import 'learning_progress.dart';

class CodeFile {
  final String filename;
  final String source;

  const CodeFile({required this.filename, required this.source});

  CodeFile copyWith({String? filename, String? source}) {
    return CodeFile(
      filename: filename ?? this.filename,
      source: source ?? this.source,
    );
  }
}

/// 当前激活的教程会话
class TutorialSession {
  final String templateKey;

  /// 模板源码语言扩展名：'c' 或 'cpp'。
  final String templateExt;
  final String generatedCode;
  final int stepIndex;
  final List<int> focusLines;
  final List<TutorialStep> steps;

  const TutorialSession({
    required this.templateKey,
    required this.templateExt,
    required this.generatedCode,
    required this.stepIndex,
    required this.focusLines,
    required this.steps,
  });

  TutorialSession copyWith({
    String? templateKey,
    String? templateExt,
    String? generatedCode,
    int? stepIndex,
    List<int>? focusLines,
    List<TutorialStep>? steps,
  }) {
    return TutorialSession(
      templateKey: templateKey ?? this.templateKey,
      templateExt: templateExt ?? this.templateExt,
      generatedCode: generatedCode ?? this.generatedCode,
      stepIndex: stepIndex ?? this.stepIndex,
      focusLines: focusLines ?? this.focusLines,
      steps: steps ?? this.steps,
    );
  }
}

class IdeState {
  final List<CodeFile> files;
  final String currentFile;
  final String source;
  final bool isCompiling;
  final bool isRunning;
  final bool isStepMode;
  final String output;
  final List<rust.Diagnostic> diagnostics;
  final List<KnowledgeCard> knowledgeCards;
  final List<rust.AlgorithmMatch> algorithmMatches;
  final List<IntentScore> intentScores;
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
  final String? activeFloatingPanel;
  final List<String> watchExpressions;
  final int executionSpeed; // 0~500 ms
  final bool showIntro;
  final LearningProgress learningProgress;
  final TutorialSession? activeTutorial;

  const IdeState({
    this.files = _defaultFiles,
    this.currentFile = 'main.c',
    this.source = _defaultCode,
    this.isCompiling = false,
    this.isRunning = false,
    this.isStepMode = false,
    this.output = '',
    this.diagnostics = const [],
    this.knowledgeCards = const [],
    this.algorithmMatches = const [],
    this.intentScores = const [],
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
    this.activeFloatingPanel,
    this.watchExpressions = const [],
    this.executionSpeed = 0,
    this.showIntro = false,
    this.learningProgress = const LearningProgress(),
    this.activeTutorial,
  });

  static const _defaultBottomSlots = ['output', 'diagnostics', 'algorithm', 'intent'];
  static const _defaultFloatingSlots = [
    'knowledge', 'pointer', 'arrayVis', 'linkedListVis', 'treeVis', 'memory', 'variables', 'watch', 'callstack', 'progress', 'varHistory',
  ];
  static const List<CodeFile> _defaultFiles = [
    CodeFile(filename: 'main.c', source: _defaultCode),
  ];

  IdeState copyWith({
    List<CodeFile>? files,
    String? currentFile,
    String? source,
    bool? isCompiling,
    bool? isRunning,
    bool? isStepMode,
    String? output,
    List<rust.Diagnostic>? diagnostics,
    List<KnowledgeCard>? knowledgeCards,
    List<rust.AlgorithmMatch>? algorithmMatches,
    List<IntentScore>? intentScores,
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
    String? activeFloatingPanel,
    List<String>? watchExpressions,
    int? executionSpeed,
    bool? showIntro,
    LearningProgress? learningProgress,
    TutorialSession? activeTutorial,
    bool clearError = false,
    bool clearActiveFloatingPanel = false,
    bool clearActiveTutorial = false,
  }) {
    return IdeState(
      files: files ?? this.files,
      currentFile: currentFile ?? this.currentFile,
      source: source ?? this.source,
      isCompiling: isCompiling ?? this.isCompiling,
      isRunning: isRunning ?? this.isRunning,
      isStepMode: isStepMode ?? this.isStepMode,
      output: output ?? this.output,
      diagnostics: diagnostics ?? this.diagnostics,
      knowledgeCards: knowledgeCards ?? this.knowledgeCards,
      algorithmMatches: algorithmMatches ?? this.algorithmMatches,
      intentScores: intentScores ?? this.intentScores,
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
      activeFloatingPanel: clearActiveFloatingPanel ? null : (activeFloatingPanel ?? this.activeFloatingPanel),
      watchExpressions: watchExpressions ?? this.watchExpressions,
      executionSpeed: executionSpeed ?? this.executionSpeed,
      showIntro: showIntro ?? this.showIntro,
      learningProgress: learningProgress ?? this.learningProgress,
      activeTutorial: clearActiveTutorial ? null : (activeTutorial ?? this.activeTutorial),
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
