import 'dart:convert';
import 'package:flutter/services.dart';
import 'code_template.dart';

/// 从 assets/templates/ 运行时加载模板列表。
///
/// 读取 assets/templates/index.json 获取元数据，
/// 再按需加载每个模板的 .c / .cpp 源码文件。
class TemplateLoader {
  /// 加载模板列表。
  ///
  /// [bundle] 用于测试注入自定义 AssetBundle；生产环境使用 [rootBundle]。
  static Future<List<CodeTemplate>> load({AssetBundle? bundle}) async {
    final assetBundle = bundle ?? rootBundle;
    final indexJson = await assetBundle.loadString('assets/templates/index.json');
    final index = jsonDecode(indexJson) as Map<String, dynamic>;
    final templates = <CodeTemplate>[];

    for (final entry in index['templates'] as List<dynamic>) {
      final meta = entry as Map<String, dynamic>;
      final key = meta['key'] as String? ?? '';
      if (key.isEmpty) continue;

      final ext = meta['ext'] as String? ?? 'c';

      // Load source code from .c / .cpp asset
      String code;
      try {
        code = await assetBundle.loadString('assets/templates/$key.$ext');
      } catch (e) {
        // 模板源码加载失败是严重错误，不应静默跳过，否则学生点击模板无反应。
        throw TemplateLoadException(
          'Failed to load template source for "$key" (expected assets/templates/$key.$ext): $e',
        );
      }

      // Parse params
      final params = <TemplateParam>[];
      final paramsMeta = meta['params'] as Map<String, dynamic>? ?? {};
      for (final e in paramsMeta.entries) {
        final p = e.value as Map<String, dynamic>? ?? {};
        final typeStr = p['type'] as String? ?? 'int';
        final paramType = typeStr == 'string'
            ? ParamType.string
            : typeStr == 'identifier'
                ? ParamType.identifier
                : ParamType.int;
        params.add(TemplateParam(
          key: e.key,
          label: p['label'] as String? ?? e.key,
          defaultValue: (p['default'] ?? '').toString(),
          type: paramType,
        ));
      }

      // Parse tutorial steps
      final tutorialSteps = <TutorialStep>[];
      final stepsMeta = meta['tutorialSteps'] as List<dynamic>? ?? [];
      for (final s in stepsMeta) {
        final step = s as Map<String, dynamic>? ?? {};
        final focusLines = (step['focusLines'] as List<dynamic>? ?? [])
            .map((v) => v as int)
            .toList();
        final explanations = <LineExplanation>[];
        final expMeta = step['explanations'] as List<dynamic>? ?? [];
        for (final e in expMeta) {
          final exp = e as Map<String, dynamic>? ?? {};
          explanations.add(LineExplanation(
            line: exp['line'] as int? ?? 0,
            short: exp['short'] as String? ?? '',
            detail: exp['detail'] as String? ?? '',
          ));
        }
        tutorialSteps.add(TutorialStep(
          title: step['title'] as String? ?? '',
          description: step['description'] as String? ?? '',
          focusLines: focusLines,
          explanations: explanations,
        ));
      }

      templates.add(CodeTemplate(
        key,
        meta['name'] as String? ?? key,
        meta['category'] as String? ?? '其他',
        code,
        ext: ext,
        params: params,
        tutorialSteps: tutorialSteps,
      ));
    }

    return templates;
  }
}

/// 模板加载异常。
class TemplateLoadException implements Exception {
  final String message;
  TemplateLoadException(this.message);
  @override
  String toString() => 'TemplateLoadException: $message';
}
