import 'package:flutter/material.dart';
import 'gutter_column.dart';
import 'gutter_context.dart';

/// ---------------------------------------------------------------------------
/// HeatmapColumn — 执行路径热力图列（4px 条带）
/// ---------------------------------------------------------------------------

class HeatmapColumn implements GutterColumn {
  @override
  double get width => 4;

  @override
  Widget buildCell(BuildContext context, int line, GutterContext ctx) {
    final count = ctx.heatmapCountForLine(line);
    final color = count > 0
        ? _heatmapColor(count / ctx.heatmapMaxCount, ctx.isDark)
        : Colors.transparent;

    return Container(
      height: 21.0,
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(1),
      ),
    );
  }

  Color _heatmapColor(double intensity, bool isDark) {
    if (intensity < 0.2) {
      return isDark ? const Color(0xFF3A3A3C) : const Color(0xFFE0E0E0);
    } else if (intensity < 0.4) {
      return isDark ? const Color(0xFF5C3A3A) : const Color(0xFFFFCDD2);
    } else if (intensity < 0.6) {
      return isDark ? const Color(0xFF7A3A3A) : const Color(0xFFEF9A9A);
    } else if (intensity < 0.8) {
      return isDark ? const Color(0xFFB04A4A) : const Color(0xFFE57373);
    } else {
      return isDark ? const Color(0xFFD32F2F) : const Color(0xFFC62828);
    }
  }
}
