import 'package:flutter/material.dart';
import 'cide_document.dart';
import 'editor_layers.dart';

/// ---------------------------------------------------------------------------
/// DiagnosticLayer — 诊断信息可视化图层
/// ---------------------------------------------------------------------------
/// 在代码行下方绘制波浪线（error=红色, warning=琥珀色, hint=蓝色）。
/// 目前仅绘制整行波浪线；精确到字符范围需要后续结合 TextPainter 的
/// getBoxesForSelection 实现。
/// ---------------------------------------------------------------------------

class DiagnosticInfo {
  final int line; // 1-based
  final int severity; // 0=error, 1=warning, 2=hint

  const DiagnosticInfo({required this.line, required this.severity});
}

class DiagnosticLayer implements EditorLayer {
  final List<DiagnosticInfo> diagnostics;

  DiagnosticLayer({this.diagnostics = const []});

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final line = layout.lineIndex + 1;
    final severity = _severityForLine(line);
    if (severity == null) return;

    final color = _severityColor(severity);
    final y = layout.top + layout.height - 2;

    _drawWavyLine(
      canvas,
      Offset(0, y),
      Offset(viewport.width, y),
      color,
    );
  }

  int? _severityForLine(int line) {
    int? result;
    for (final d in diagnostics) {
      if (d.line == line) {
        if (result == null || d.severity < result) {
          result = d.severity;
        }
      }
    }
    return result;
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
