import 'package:flutter/material.dart';
import '../models/code_template.dart';
import 'template_chip.dart';

class TemplateBar extends StatelessWidget {
  final List<CodeTemplate> templates;
  final void Function(CodeTemplate template) onSelectTemplate;

  const TemplateBar({
    super.key,
    required this.templates,
    required this.onSelectTemplate,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 36,
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
        ),
      ),
      child: ListView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 4),
        children: templates.map((tmpl) {
          return TemplateChip(
            label: tmpl.displayName,
            onTap: () => onSelectTemplate(tmpl),
          );
        }).toList(),
      ),
    );
  }
}
