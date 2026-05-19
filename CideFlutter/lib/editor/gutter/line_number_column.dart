import 'package:flutter/material.dart';
import 'gutter_column.dart';
import 'gutter_context.dart';

/// ---------------------------------------------------------------------------
/// LineNumberColumn — 行号列（含断点、诊断、执行行、变量访问标记）
/// ---------------------------------------------------------------------------

class LineNumberColumn implements GutterColumn {
  @override
  double get width => 64;

  @override
  Widget buildCell(BuildContext context, int line, GutterContext ctx) {
    final hasBreakpoint = ctx.breakpoints.contains(line);
    final severity = ctx.severityForLine(line);

    String prefix = '';
    if (hasBreakpoint) {
      prefix = '● ';
    } else if (severity == 0) {
      prefix = '✗ ';
    } else if (severity == 1) {
      prefix = '⚠ ';
    } else if (severity == 2) {
      prefix = 'ℹ ';
    }

    if (ctx.isStepMode && line == ctx.currentDebugLine) {
      prefix = '$prefix▶ ';
    }

    String varSuffix = '';
    if (ctx.accessedVars.isNotEmpty) {
      final markers = ctx.accessedVars
          .where((a) => false) // 由外层统一计算当前行的变量标记
          .take(2)
          .map((a) => '${a.name}=${a.accessType == 'Read' ? 'R' : 'W'}')
          .join(' ');
      if (markers.isNotEmpty) varSuffix = ' $markers';
    }

    final isCurrentLine = line == ctx.currentLine;
    final lineNumberColor = ctx.isDark
        ? const Color(0xff5c6370)
        : const Color(0xffa0a1a7);
    final focusedLineNumberColor = ctx.isDark
        ? const Color(0xffabb2bf)
        : const Color(0xff383a42);

    return Container(
      height: 21.0,
      alignment: Alignment.centerRight,
      padding: const EdgeInsets.only(right: 8),
      child: Text(
        prefix.isEmpty ? '$line$varSuffix' : '$prefix$line$varSuffix',
        style: TextStyle(
          fontSize: 14,
          color: isCurrentLine ? focusedLineNumberColor : lineNumberColor,
          fontFamily: 'monospace',
        ),
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
    );
  }
}
