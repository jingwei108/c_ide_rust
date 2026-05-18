import 'dart:convert';

/// 学习进度数据模型
///
/// 追踪用户在 IDE 中的学习行为：编译历史、错误修复、知识卡片阅读、算法验证等。
class LearningProgress {
  final int totalCompiles;
  final int successfulCompiles;
  final int failedCompiles;
  final Map<String, int> errorsByCode;
  final Map<String, int> fixedErrorsByCode;
  final Set<String> viewedKnowledgeCards;
  final Map<String, int> algorithmValidationsPassed;
  final Map<String, int> algorithmValidationsTotal;
  final String lastActiveDate;
  final int streakDays;
  final int totalUnifiedRuns;
  final int totalStepsExecuted;
  final int totalTraps;
  final int totalSeeks;
  final int maxStepsInSingleRun;
  final Set<String> completedTutorials;

  const LearningProgress({
    this.totalCompiles = 0,
    this.successfulCompiles = 0,
    this.failedCompiles = 0,
    this.errorsByCode = const {},
    this.fixedErrorsByCode = const {},
    this.viewedKnowledgeCards = const {},
    this.algorithmValidationsPassed = const {},
    this.algorithmValidationsTotal = const {},
    this.lastActiveDate = '',
    this.streakDays = 0,
    this.totalUnifiedRuns = 0,
    this.totalStepsExecuted = 0,
    this.totalTraps = 0,
    this.totalSeeks = 0,
    this.maxStepsInSingleRun = 0,
    this.completedTutorials = const {},
  });

  double get successRate => totalCompiles == 0 ? 0.0 : successfulCompiles / totalCompiles;

  int get totalErrorsEncountered => errorsByCode.values.fold(0, (a, b) => a + b);
  int get totalErrorsFixed => fixedErrorsByCode.values.fold(0, (a, b) => a + b);

  double get algorithmOverallPassRate {
    final total = algorithmValidationsTotal.values.fold(0, (a, b) => a + b);
    if (total == 0) return 0.0;
    final passed = algorithmValidationsPassed.values.fold(0, (a, b) => a + b);
    return passed / total;
  }

  LearningProgress copyWith({
    int? totalCompiles,
    int? successfulCompiles,
    int? failedCompiles,
    Map<String, int>? errorsByCode,
    Map<String, int>? fixedErrorsByCode,
    Set<String>? viewedKnowledgeCards,
    Map<String, int>? algorithmValidationsPassed,
    Map<String, int>? algorithmValidationsTotal,
    String? lastActiveDate,
    int? streakDays,
    int? totalUnifiedRuns,
    int? totalStepsExecuted,
    int? totalTraps,
    int? totalSeeks,
    int? maxStepsInSingleRun,
    Set<String>? completedTutorials,
  }) {
    return LearningProgress(
      totalCompiles: totalCompiles ?? this.totalCompiles,
      successfulCompiles: successfulCompiles ?? this.successfulCompiles,
      failedCompiles: failedCompiles ?? this.failedCompiles,
      errorsByCode: errorsByCode ?? this.errorsByCode,
      fixedErrorsByCode: fixedErrorsByCode ?? this.fixedErrorsByCode,
      viewedKnowledgeCards: viewedKnowledgeCards ?? this.viewedKnowledgeCards,
      algorithmValidationsPassed: algorithmValidationsPassed ?? this.algorithmValidationsPassed,
      algorithmValidationsTotal: algorithmValidationsTotal ?? this.algorithmValidationsTotal,
      lastActiveDate: lastActiveDate ?? this.lastActiveDate,
      streakDays: streakDays ?? this.streakDays,
      totalUnifiedRuns: totalUnifiedRuns ?? this.totalUnifiedRuns,
      totalStepsExecuted: totalStepsExecuted ?? this.totalStepsExecuted,
      totalTraps: totalTraps ?? this.totalTraps,
      totalSeeks: totalSeeks ?? this.totalSeeks,
      maxStepsInSingleRun: maxStepsInSingleRun ?? this.maxStepsInSingleRun,
      completedTutorials: completedTutorials ?? this.completedTutorials,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'totalCompiles': totalCompiles,
      'successfulCompiles': successfulCompiles,
      'failedCompiles': failedCompiles,
      'errorsByCode': errorsByCode,
      'fixedErrorsByCode': fixedErrorsByCode,
      'viewedKnowledgeCards': viewedKnowledgeCards.toList(),
      'algorithmValidationsPassed': algorithmValidationsPassed,
      'algorithmValidationsTotal': algorithmValidationsTotal,
      'lastActiveDate': lastActiveDate,
      'streakDays': streakDays,
      'totalUnifiedRuns': totalUnifiedRuns,
      'totalStepsExecuted': totalStepsExecuted,
      'totalTraps': totalTraps,
      'totalSeeks': totalSeeks,
      'maxStepsInSingleRun': maxStepsInSingleRun,
      'completedTutorials': completedTutorials.toList(),
    };
  }

  factory LearningProgress.fromJson(Map<String, dynamic> json) {
    return LearningProgress(
      totalCompiles: json['totalCompiles'] as int? ?? 0,
      successfulCompiles: json['successfulCompiles'] as int? ?? 0,
      failedCompiles: json['failedCompiles'] as int? ?? 0,
      errorsByCode: (json['errorsByCode'] as Map<String, dynamic>?)?.map(
            (k, v) => MapEntry(k, v as int),
          ) ??
          const {},
      fixedErrorsByCode: (json['fixedErrorsByCode'] as Map<String, dynamic>?)?.map(
            (k, v) => MapEntry(k, v as int),
          ) ??
          const {},
      viewedKnowledgeCards: (json['viewedKnowledgeCards'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toSet() ??
          const {},
      algorithmValidationsPassed:
          (json['algorithmValidationsPassed'] as Map<String, dynamic>?)?.map(
                (k, v) => MapEntry(k, v as int),
              ) ??
              const {},
      algorithmValidationsTotal:
          (json['algorithmValidationsTotal'] as Map<String, dynamic>?)?.map(
                (k, v) => MapEntry(k, v as int),
              ) ??
              const {},
      lastActiveDate: json['lastActiveDate'] as String? ?? '',
      streakDays: json['streakDays'] as int? ?? 0,
      totalUnifiedRuns: json['totalUnifiedRuns'] as int? ?? 0,
      totalStepsExecuted: json['totalStepsExecuted'] as int? ?? 0,
      totalTraps: json['totalTraps'] as int? ?? 0,
      totalSeeks: json['totalSeeks'] as int? ?? 0,
      maxStepsInSingleRun: json['maxStepsInSingleRun'] as int? ?? 0,
      completedTutorials: (json['completedTutorials'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toSet() ??
          const {},
    );
  }

  String toJsonString() => jsonEncode(toJson());

  factory LearningProgress.fromJsonString(String s) {
    return LearningProgress.fromJson(jsonDecode(s) as Map<String, dynamic>);
  }
}
