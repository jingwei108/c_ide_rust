import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../models/template_registry.dart';
import '../../providers/ide_provider.dart';
import '../../widgets/template_bar.dart';
import '../../widgets/template_param_dialog.dart';

/// IDE 模板快捷栏包装组件。
///
/// 负责从 assets 异步加载模板列表，处理模板选择、参数对话框与教程启动，
/// 并在键盘弹出时通过 [animation] 平滑收起。
class IdeTemplateBar extends ConsumerWidget {
  final Animation<double> animation;
  final void Function(String text) onInsertText;
  final void Function(int line) onScrollToLine;

  const IdeTemplateBar({
    super.key,
    required this.animation,
    required this.onInsertText,
    required this.onScrollToLine,
  });

  void _handleTemplateSelect(BuildContext context, WidgetRef ref, CodeTemplate template) {
    final notifier = ref.read(ideProvider.notifier);

    void scrollToTutorialFocus() {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!context.mounted) return;
        final tutorial = ref.read(ideProvider).activeTutorial;
        if (tutorial != null && tutorial.focusLines.isNotEmpty) {
          onScrollToLine(tutorial.focusLines.first);
        }
      });
    }

    // 无参数且无教程：直接插入（旧行为）
    if (template.params.isEmpty && template.tutorialSteps.isEmpty) {
      onInsertText(template.code);
      return;
    }

    // 有参数：先弹参数对话框
    if (template.params.isNotEmpty) {
      showTemplateParamDialog(
        context: context,
        template: template,
        onConfirm: (params) {
          final generated = template.buildCode(params);
          if (template.tutorialSteps.isNotEmpty) {
            // 启动教程
            notifier.startTutorial(template, generated);
            scrollToTutorialFocus();
          } else {
            // 无教程，直接插入
            onInsertText(generated);
          }
        },
      );
      return;
    }

    // 无参数但有教程
    notifier.startTutorial(template, template.code);
    scrollToTutorialFocus();
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return SizeTransition(
      sizeFactor: animation,
      axisAlignment: 1,
      child: FutureBuilder<List<CodeTemplate>>(
        future: getDynamicTemplates(),
        builder: (context, snapshot) {
          if (!snapshot.hasData) {
            return const SizedBox.shrink();
          }
          return TemplateBar(
            templates: snapshot.data!,
            onSelectTemplate: (template) => _handleTemplateSelect(context, ref, template),
          );
        },
      ),
    );
  }
}
