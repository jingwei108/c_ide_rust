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

  /// 源码语言扩展名：'c' 或 'cpp'。
  /// 用于插入模板时设置正确的文件名，确保 Rust 后端按 C/C++ 模式编译。
  final String ext;
  final List<TemplateParam> params;
  final List<TutorialStep> tutorialSteps;

  const CodeTemplate(
    this.key,
    this.displayName,
    this.category,
    this.code, {
    this.ext = 'c',
    this.params = const [],
    this.tutorialSteps = const [],
  });

  /// 占位符语法：/*__PARAM_key__*/ defaultValue
  ///
  /// defaultValue 匹配规则：从占位符后开始，直到遇到空白、逗号、分号、
  /// 右括号、右中括号或字符串结束。这样可以覆盖数组大小、函数参数、
  /// 变量初始化等常见场景。
  static final _placeholderRe = RegExp(r'/\*__PARAM_(\w+)__\*/\s*([^\s\[\]();,]+)');

  /// 用学生填入的参数替换代码中的占位符。
  String buildCode(Map<String, String> values) {
    return code.replaceAllMapped(_placeholderRe, (match) {
      final paramKey = match.group(1)!;
      final paramDefault = match.group(2)!;
      return values[paramKey] ?? paramDefault;
    });
  }

  /// 返回该模板源码中声明的所有占位符参数名。
  Set<String> get placeholderKeys {
    return _placeholderRe.allMatches(code).map((m) => m.group(1)!).toSet();
  }
}
