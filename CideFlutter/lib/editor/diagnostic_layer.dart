import 'package:flutter/material.dart';
import 'cide_document.dart';
import 'editor_layers.dart';

/// ---------------------------------------------------------------------------
/// DiagnosticLayer — 诊断信息可视化图层（字符级精确波浪线）
/// ---------------------------------------------------------------------------
/// 在诊断范围的字符下方绘制精确波浪线。
/// 支持一行多个诊断、不同严重级别颜色区分。
/// ---------------------------------------------------------------------------

class DiagnosticInfo {
  final int line; // 1-based
  final int severity; // 0=error, 1=warning, 2=hint
  final int startCol; // 0-based, inclusive
  final int endCol;   // 0-based, exclusive

  const DiagnosticInfo({
    required this.line,
    required this.severity,
    this.startCol = 0,
    this.endCol = 0,
  });
}

class DiagnosticLayer implements EditorLayer {
  final List<DiagnosticInfo> diagnostics;

  DiagnosticLayer({this.diagnostics = const []});

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final line = layout.lineIndex + 1;

    // 收集当前行的所有诊断（按 startCol 排序）
    final lineDiagnostics = <DiagnosticInfo>[];
    for (final d in diagnostics) {
      if (d.line == line) {
        lineDiagnostics.add(d);
      }
    }
    if (lineDiagnostics.isEmpty) return;

    lineDiagnostics.sort((a, b) => a.startCol.compareTo(b.startCol));

    final textLength = layout.text.length;

    for (final d in lineDiagnostics) {
      final start = d.startCol.clamp(0, textLength);
      final end = d.endCol.clamp(start, textLength);
      if (start >= end) continue;

      final color = _severityColor(d.severity);
      final boxes = layout.painter.getBoxesForSelection(
        TextSelection(baseOffset: start, extentOffset: end),
      );

      for (final box in boxes) {
        final rect = box.toRect().translate(0, layout.top);
        final y = rect.bottom - 1.5;
        _drawWavyLine(
          canvas,
          Offset(rect.left, y),
          Offset(rect.right, y),
          color,
        );
      }
    }
  }

  Color _severityColor(int severity) {
    switch (severity) {
      case 0:
        return const Color(0xFFF44336); // error: red
      case 1:
        return const Color(0xFFFF9800); // warning: amber
      case 2:
        return const Color(0xFF2196F3); // hint: blue
      default:
        return Colors.grey;
    }
  }

  /// 绘制波浪线
  void _drawWavyLine(Canvas canvas, Offset start, Offset end, Color color) {
    final paint = Paint()
      ..color = color
      ..strokeWidth = 1.5
      ..style = PaintingStyle.stroke;

    final path = Path();
    path.moveTo(start.dx, start.dy);

    const waveWidth = 6.0;
    const waveHeight = 2.5;
    double x = start.dx;
    bool up = true;

    while (x < end.dx) {
      final nextX = (x + waveWidth).clamp(start.dx, end.dx);
      final midX = (x + nextX) / 2;
      final controlY = up ? start.dy - waveHeight : start.dy + waveHeight;
      path.quadraticBezierTo(midX, controlY, nextX, start.dy);
      x = nextX;
      up = !up;
    }

    canvas.drawPath(path, paint);
  }
}
