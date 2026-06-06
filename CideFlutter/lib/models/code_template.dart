/// 参数类型
enum ParamType { int, string, identifier }

/// 模板参数定义
class TemplateParam {
  final String key;
  final String label;
  final String defaultValue;
  final ParamType type;

  const TemplateParam({
    required this.key,
    required this.label,
    required this.defaultValue,
    this.type = ParamType.int,
  });
}

/// 单行代码解释
class LineExplanation {
  final int line;
  final String short;
  final String detail;

  const LineExplanation({
    required this.line,
    required this.short,
    required this.detail,
  });
}

/// 教程步骤
class TutorialStep {
  final String title;
  final String description;
  final List<int> focusLines;
  final List<LineExplanation> explanations;

  const TutorialStep({
    required this.title,
    required this.description,
    required this.focusLines,
    this.explanations = const [],
  });
}

/// 代码模板
class CodeTemplate {
  final String key;
  final String displayName;
  final String category;
  final String code;
  final List<TemplateParam> params;
  final List<TutorialStep> tutorialSteps;

  const CodeTemplate(
    this.key,
    this.displayName,
    this.category,
    this.code, {
    this.params = const [],
    this.tutorialSteps = const [],
  });

  /// 用学生填入的参数替换代码中的占位符。
  /// 占位符语法 (合法C注释): /*__PARAM_key__*/ defaultValue
  String buildCode(Map<String, String> values) {
    var result = code;
    final pattern = RegExp(r'/\*__PARAM_(\w+)__\*/\s*([^ \t\n\r\[\]();,]+)');
    result = result.replaceAllMapped(pattern, (match) {
      final paramKey = match.group(1)!;
      final paramDefault = match.group(2)!;
      return values[paramKey] ?? paramDefault;
    });
    return result;
  }
}
