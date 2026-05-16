import 'package:cide/src/rust/api/types.dart' as rust;
import 'knowledge_card.dart';
import 'learning_progress.dart';

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
  final String? activeFloatingPanel;
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
    this.activeFloatingPanel,
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
    String? activeFloatingPanel,
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
      activeFloatingPanel: activeFloatingPanel ?? this.activeFloatingPanel,
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
