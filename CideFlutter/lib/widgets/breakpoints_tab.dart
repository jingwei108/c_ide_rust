import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/ide_provider.dart';

class BreakpointsTab extends ConsumerWidget {
  final IdeState state;
  final bool isDark;
  final void Function(int line)? onScrollToLine;

  const BreakpointsTab({super.key, required this.state, required this.isDark, this.onScrollToLine});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notifier = ref.read(ideProvider.notifier);
    final breakpoints = state.breakpoints.toList()..sort();

    if (breakpoints.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.stop_circle_outlined, size: 40, color: Colors.grey[500]),
            const SizedBox(height: 12),
            Text('暂无断点', style: TextStyle(fontSize: 14, color: Colors.grey[500])),
            const SizedBox(height: 4),
            Text('在编辑器左侧行号旁点击可添加断点', style: TextStyle(fontSize: 12, color: Colors.grey[500])),
          ],
        ),
      );
    }

    final sourceLines = state.source.split('\n');

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: breakpoints.length,
      itemBuilder: (context, index) {
        final line = breakpoints[index];
        final lineText = line > 0 && line <= sourceLines.length
            ? sourceLines[line - 1].trim()
            : '';

        return InkWell(
          onTap: () => onScrollToLine?.call(line),
          borderRadius: BorderRadius.circular(6),
          child: Container(
            margin: const EdgeInsets.only(bottom: 6),
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: isDark ? const Color(0xff1e1e1e) : const Color(0xfff5f5f5),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(
                color: isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5),
              ),
            ),
            child: Row(
              children: [
              const Icon(Icons.stop_circle, color: Colors.redAccent, size: 16),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '第 $line 行',
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                        color: isDark ? const Color(0xffd4d4d4) : const Color(0xff383a42),
                      ),
                    ),
                    if (lineText.isNotEmpty)
                      Text(
                        lineText,
                        style: TextStyle(
                          fontSize: 11,
                          color: isDark ? const Color(0xff5c6370) : const Color(0xffa0a1a7),
                          fontFamily: 'monospace',
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                  ],
                ),
              ),
              IconButton(
                icon: const Icon(Icons.close, size: 18),
                color: Colors.grey[500],
                tooltip: '删除断点',
                onPressed: () => notifier.toggleBreakpoint(line),
              ),
            ],
          ),
        ),
      );
      },
    );
  }
}
