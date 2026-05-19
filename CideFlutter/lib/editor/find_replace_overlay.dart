import 'package:flutter/material.dart';
import 'find_replace_controller.dart';

/// ---------------------------------------------------------------------------
/// FindReplaceOverlay — 查找替换浮动面板
/// ---------------------------------------------------------------------------
/// 显示在编辑器右上角，支持查找/替换/导航。
/// ---------------------------------------------------------------------------

class FindReplaceOverlay extends StatelessWidget {
  final FindReplaceController controller;
  final VoidCallback onFindNext;
  final VoidCallback onFindPrevious;
  final VoidCallback onReplace;
  final VoidCallback onReplaceAll;
  final VoidCallback onClose;
  final bool isDark;

  const FindReplaceOverlay({
    super.key,
    required this.controller,
    required this.onFindNext,
    required this.onFindPrevious,
    required this.onReplace,
    required this.onReplaceAll,
    required this.onClose,
    required this.isDark,
  });

  @override
  Widget build(BuildContext context) {
    final bgColor = isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff);
    final borderColor = isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5);
    final textColor = isDark ? const Color(0xffd4d4d4) : const Color(0xff383a42);

    return Material(
      elevation: 6,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        width: 380,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: bgColor,
          border: Border.all(color: borderColor),
          borderRadius: BorderRadius.circular(4),
        ),
        child: AnimatedBuilder(
          animation: controller,
          builder: (context, child) {
            return Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 查找行
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        autofocus: true,
                        style: TextStyle(color: textColor, fontSize: 13),
                        decoration: InputDecoration(
                          hintText: '查找',
                          hintStyle: TextStyle(color: textColor.withValues(alpha: 0.4)),
                          isDense: true,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(4),
                            borderSide: BorderSide(color: borderColor),
                          ),
                        ),
                        onChanged: controller.setQuery,
                        onSubmitted: (_) => onFindNext(),
                      ),
                    ),
                    const SizedBox(width: 8),
                    _IconButton(icon: Icons.arrow_upward, tooltip: '上一个', onTap: onFindPrevious),
                    _IconButton(icon: Icons.arrow_downward, tooltip: '下一个', onTap: onFindNext),
                    _IconButton(icon: Icons.close, tooltip: '关闭', onTap: onClose),
                  ],
                ),
                const SizedBox(height: 8),
                // 替换行
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        style: TextStyle(color: textColor, fontSize: 13),
                        decoration: InputDecoration(
                          hintText: '替换',
                          hintStyle: TextStyle(color: textColor.withValues(alpha: 0.4)),
                          isDense: true,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(4),
                            borderSide: BorderSide(color: borderColor),
                          ),
                        ),
                        onChanged: controller.setReplacement,
                        onSubmitted: (_) => onReplace(),
                      ),
                    ),
                    const SizedBox(width: 8),
                    _TextButton(text: '替换', onTap: onReplace),
                    _TextButton(text: '全部', onTap: onReplaceAll),
                  ],
                ),
                const SizedBox(height: 8),
                // 选项行
                Row(
                  children: [
                    _ToggleButton(
                      text: 'Aa',
                      active: controller.caseSensitive,
                      onTap: controller.toggleCaseSensitive,
                    ),
                    const SizedBox(width: 8),
                    _ToggleButton(
                      text: '.*',
                      active: controller.useRegex,
                      onTap: controller.toggleRegex,
                    ),
                    const Spacer(),
                    if (controller.hasMatches)
                      Text(
                        '${controller.currentMatchIndex + 1} / ${controller.matches.length}',
                        style: TextStyle(fontSize: 12, color: textColor.withValues(alpha: 0.6)),
                      ),
                  ],
                ),
              ],
            );
          },
        ),
      ),
    );
  }
}

class _IconButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback onTap;

  const _IconButton({required this.icon, required this.tooltip, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Padding(
          padding: const EdgeInsets.all(6),
          child: Icon(icon, size: 16),
        ),
      ),
    );
  }
}

class _TextButton extends StatelessWidget {
  final String text;
  final VoidCallback onTap;

  const _TextButton({required this.text, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        child: Text(text, style: const TextStyle(fontSize: 12)),
      ),
    );
  }
}

class _ToggleButton extends StatelessWidget {
  final String text;
  final bool active;
  final VoidCallback onTap;

  const _ToggleButton({required this.text, required this.active, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: active ? Colors.blueAccent.withValues(alpha: 0.3) : null,
          borderRadius: BorderRadius.circular(4),
        ),
        child: Text(
          text,
          style: TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.bold,
            color: active ? Colors.blueAccent : null,
          ),
        ),
      ),
    );
  }
}
