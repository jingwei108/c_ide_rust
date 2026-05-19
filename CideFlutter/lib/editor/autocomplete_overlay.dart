import 'package:flutter/material.dart';
import 'autocomplete_controller.dart';

/// ---------------------------------------------------------------------------
/// AutocompleteOverlay — 自动补全候选列表浮层
/// ---------------------------------------------------------------------------
/// 显示在光标下方，支持点击选择和键盘导航。
/// ---------------------------------------------------------------------------

class AutocompleteOverlay extends StatelessWidget {
  final AutocompleteController controller;
  final VoidCallback onDismiss;
  final ValueChanged<AutocompleteCandidate> onSelected;
  final bool isDark;

  const AutocompleteOverlay({
    super.key,
    required this.controller,
    required this.onDismiss,
    required this.onSelected,
    required this.isDark,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, child) {
        if (!controller.visible || controller.candidates.isEmpty) {
          return const SizedBox.shrink();
        }

        final bgColor = isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff);
        final borderColor = isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5);
        final textColor = isDark ? const Color(0xffd4d4d4) : const Color(0xff383a42);

        return Material(
          elevation: 4,
          borderRadius: BorderRadius.circular(4),
          color: bgColor,
          child: Container(
            constraints: const BoxConstraints(maxHeight: 200, maxWidth: 240),
            decoration: BoxDecoration(
              border: Border.all(color: borderColor),
              borderRadius: BorderRadius.circular(4),
            ),
            child: ListView.builder(
              shrinkWrap: true,
              itemCount: controller.candidates.length,
              itemBuilder: (context, index) {
                final candidate = controller.candidates[index];
                final isSelected = index == controller.selectedIndex;
                return InkWell(
                  onTap: () {
                    controller.selectedIndex = index; // 需要 setter
                    onSelected(controller.confirm()!);
                  },
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                    decoration: BoxDecoration(
                      color: isSelected
                          ? Colors.blueAccent.withValues(alpha: 0.3)
                          : null,
                    ),
                    child: Row(
                      children: [
                        _TypeIcon(type: candidate.type),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            candidate.word,
                            style: TextStyle(
                              fontSize: 13,
                              color: isSelected ? Colors.white : textColor,
                              fontFamily: 'monospace',
                            ),
                          ),
                        ),
                        if (candidate.signature != null)
                          Text(
                            candidate.signature!,
                            style: const TextStyle(
                              fontSize: 10,
                              color: Colors.grey,
                              fontFamily: 'monospace',
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
        );
      },
    );
  }
}

class _TypeIcon extends StatelessWidget {
  final String? type;

  const _TypeIcon({this.type});

  @override
  Widget build(BuildContext context) {
    final color = switch (type) {
      'function' => Colors.yellowAccent,
      'macro' => Colors.cyanAccent,
      _ => Colors.blueAccent,
    };
    final icon = switch (type) {
      'function' => Icons.functions,
      'macro' => Icons.data_object,
      _ => Icons.text_fields,
    };
    return Icon(icon, size: 14, color: color);
  }
}
