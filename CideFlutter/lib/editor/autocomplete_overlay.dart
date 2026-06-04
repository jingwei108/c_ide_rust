import 'package:flutter/material.dart';
import 'autocomplete_controller.dart';

/// ---------------------------------------------------------------------------
/// AutocompleteOverlay — 自动补全候选列表浮层（v2 语义增强）
/// ---------------------------------------------------------------------------
/// 显示在光标下方，支持点击选择和键盘导航。
/// 增强：更多类型图标、语义候选 detail 显示、加载指示器。
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
        if (!controller.visible && !controller.fetchingSemantic) {
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
            constraints: const BoxConstraints(maxHeight: 240, maxWidth: 280),
            decoration: BoxDecoration(
              border: Border.all(color: borderColor),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (controller.fetchingSemantic && controller.candidates.isEmpty)
                  Padding(
                    padding: const EdgeInsets.all(12),
                    child: Row(
                      children: [
                        SizedBox(
                          width: 14,
                          height: 14,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: isDark ? Colors.grey : Colors.blueAccent,
                          ),
                        ),
                        const SizedBox(width: 8),
                        Text(
                          '分析中...',
                          style: TextStyle(
                            fontSize: 12,
                            color: isDark ? Colors.grey : Colors.grey.shade600,
                          ),
                        ),
                      ],
                    ),
                  ),
                Flexible(
                  child: ListView.builder(
                    shrinkWrap: true,
                    padding: EdgeInsets.zero,
                    itemCount: controller.candidates.length,
                    itemBuilder: (context, index) {
                      final candidate = controller.candidates[index];
                      final isSelected = index == controller.selectedIndex;
                      return InkWell(
                        onTap: () {
                          controller.selectedIndex = index;
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
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    Text(
                                      candidate.word,
                                      style: TextStyle(
                                        fontSize: 13,
                                        color: isSelected ? Colors.white : textColor,
                                        fontFamily: 'monospace',
                                      ),
                                    ),
                                    if (candidate.detail != null && candidate.detail!.isNotEmpty)
                                      Text(
                                        candidate.detail!,
                                        style: TextStyle(
                                          fontSize: 10,
                                          color: isSelected
                                              ? Colors.white70
                                              : Colors.grey.shade500,
                                          fontFamily: 'monospace',
                                        ),
                                        maxLines: 1,
                                        overflow: TextOverflow.ellipsis,
                                      ),
                                  ],
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
              ],
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
    final (color, icon) = switch (type) {
      'function' => (Colors.yellowAccent, Icons.functions),
      'macro' => (Colors.cyanAccent, Icons.data_object),
      'variable' => (Colors.lightGreenAccent, Icons.adjust),
      'struct' => (Colors.orangeAccent, Icons.account_tree),
      'union' => (Colors.deepOrangeAccent, Icons.account_tree),
      'enum' => (Colors.purpleAccent, Icons.format_list_numbered),
      'typedef' => (Colors.tealAccent, Icons.short_text),
      'field' => (Colors.pinkAccent, Icons.arrow_right),
      'snippet' => (Colors.amberAccent, Icons.code),
      'format' => (Colors.indigoAccent, Icons.percent),
      _ => (Colors.blueAccent, Icons.text_fields),
    };
    return Icon(icon, size: 14, color: color);
  }
}
