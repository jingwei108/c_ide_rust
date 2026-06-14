import 'package:flutter/material.dart';
import 'gutter_column.dart';
import 'gutter_context.dart';

/// ---------------------------------------------------------------------------
/// GutterView — Gutter 组合视图（懒加载）
/// ---------------------------------------------------------------------------
/// 只渲染可见行，根据 scrollOffset 和 viewportHeight 计算可见范围。
/// ---------------------------------------------------------------------------

class GutterView extends StatelessWidget {
  final List<GutterColumn> columns;
  final GutterContext context;
  final double scrollOffset;
  final double viewportHeight;
  final double lineHeight;
  final int lineCount;
  final VoidCallback? onTapLine;

  const GutterView({
    super.key,
    required this.columns,
    required this.context,
    required this.scrollOffset,
    required this.viewportHeight,
    required this.lineHeight,
    required this.lineCount,
    this.onTapLine,
  });

  @override
  Widget build(BuildContext buildContext) {
    final firstVisible = (scrollOffset / lineHeight).floor().clamp(0, lineCount - 1);
    final lastVisible = ((scrollOffset + viewportHeight) / lineHeight).ceil().clamp(0, lineCount - 1);

    // 上下各多渲染 2 行，避免快速滚动时出现空白
    final renderStart = (firstVisible - 2).clamp(0, lineCount - 1);
    final renderEnd = (lastVisible + 2).clamp(0, lineCount - 1);

    final totalWidth = columns.fold<double>(0.0, (sum, col) => sum + col.width);

    return Container(
      width: totalWidth,
      color: context.isDark ? const Color(0xff21252b) : const Color(0xfff0f0f0),
      child: ClipRect(
        child: OverflowBox(
          maxWidth: totalWidth,
          maxHeight: double.infinity,
          alignment: Alignment.topCenter,
          child: Transform.translate(
            offset: Offset(0, -scrollOffset),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                for (final col in columns)
                  SizedBox(
                    width: col.width,
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        // 上方占位（未渲染行）
                        SizedBox(height: renderStart * lineHeight),
                        // 可见行
                        for (int i = renderStart; i <= renderEnd; i++)
                          GestureDetector(
                            onTap: onTapLine != null ? () => onTapLine!() : null,
                            child: col.buildCell(buildContext, i + 1, context),
                          ),
                        // 下方占位（未渲染行）
                        SizedBox(height: (lineCount - 1 - renderEnd) * lineHeight),
                      ],
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
