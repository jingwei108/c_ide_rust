import 'dart:convert';
import 'package:flutter/services.dart' show rootBundle;
import 'code_template.dart';

/// 从 assets/templates/ 运行时加载模板列表。
///
/// 读取 assets/templates/index.json 获取元数据，
/// 再按需加载每个模板的 .c 源码文件。
class TemplateLoader {
  static Future<List<CodeTemplate>> load() async {
    final indexJson = await rootBundle.loadString('assets/templates/index.json');
    final index = jsonDecode(indexJson) as Map<String, dynamic>;
    final templates = <CodeTemplate>[];

    for (final entry in index['templates'] as List<dynamic>) {
      final meta = entry as Map<String, dynamic>;
      final key = meta['key'] as String? ?? '';
      if (key.isEmpty) continue;

      // Load source code from .c asset
      String code;
      try {
        code = await rootBundle.loadString('assets/templates/$key.c');
      } catch (_) {
        // Fallback: if .c file missing, skip this template
        continue;
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
        params: params,
        tutorialSteps: tutorialSteps,
      ));
    }

    return templates;
  }
}
