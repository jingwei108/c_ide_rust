import 'package:flutter/material.dart';
import 'cide_document.dart';

/// ---------------------------------------------------------------------------
/// EditorLayer — 编辑器图层接口
/// ---------------------------------------------------------------------------
/// 每个图层独立负责一类视觉元素的绘制：
/// - TextLayer：纯文本（带语法高亮）
/// - SelectionLayer：选区背景 + 光标
/// - ComposingLayer：IME composing 下划线
/// - DiagnosticLayer：诊断波浪线 / 错误背景
/// - RuntimeLayer：执行行高亮 / 变量访问高亮
/// - TutorialLayer：教程行高亮
/// ---------------------------------------------------------------------------

/// 单行布局信息（由 CideEditorPainter 预先计算并缓存）
class LineLayout {
  final int lineIndex;
  final String text;
  final double top;
  final double height;
  final TextPainter painter;

  LineLayout({
    required this.lineIndex,
    required this.text,
    required this.top,
    required this.height,
    required this.painter,
  });
}

abstract class EditorLayer {
  /// 绘制该图层。
  /// [canvas]：画布
  /// [layout]：当前行的布局信息
  /// [document]：完整文档
  /// [viewport]：当前可见区域（相对坐标）
  void paint(
    Canvas canvas,
    LineLayout layout,
    CideDocument document,
    Rect viewport,
  );
}

// ---------------------------------------------------------------------------
// TextLayer — 纯文本绘制（带基础语法高亮占位）
// ---------------------------------------------------------------------------
class TextLayer implements EditorLayer {
  final TextStyle baseStyle;
  final Map<String, TextStyle> keywordStyles;

  TextLayer({
    required this.baseStyle,
    this.keywordStyles = const {},
  });

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    // 直接绘制预先生成的 TextPainter
    layout.painter.paint(canvas, Offset(0, layout.top));
  }
}

// ---------------------------------------------------------------------------
// SelectionLayer — 选区背景 + 光标
// ---------------------------------------------------------------------------
class SelectionLayer implements EditorLayer {
  final Color selectionColor;
  final Color cursorColor;
  final double cursorWidth;
  final bool cursorVisible;

  SelectionLayer({
    this.selectionColor = const Color(0x6680CBC4),
    this.cursorColor = Colors.white,
    this.cursorWidth = 2.0,
    this.cursorVisible = true,
  });

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final sel = document.selection;
    final line = layout.lineIndex;

    // 判断当前行是否与选区相交
    final start = sel.start;
    final end = sel.end;

    // 选区背景
    if (start.line <= line && line <= end.line && !sel.isCollapsed) {
      int colStart = 0;
      int colEnd = layout.text.length;

      if (line == start.line) colStart = start.col;
      if (line == end.line) colEnd = end.col;

      final boxes = layout.painter.getBoxesForSelection(
        TextSelection(baseOffset: colStart, extentOffset: colEnd),
      );
      for (final box in boxes) {
        final rect = box.toRect().translate(0, layout.top);
        canvas.drawRect(rect, Paint()..color = selectionColor);
      }
    }

    // 光标（collapsed 或当前行包含光标）
    if (cursorVisible && sel.isCollapsed && sel.base.line == line) {
      final offset = layout.painter.getOffsetForCaret(
        TextPosition(offset: sel.base.col),
        Rect.zero,
      );
      final cursorRect = Rect.fromLTWH(
        offset.dx,
        layout.top,
        cursorWidth,
        layout.height,
      );
      canvas.drawRect(cursorRect, Paint()..color = cursorColor);
    }
  }
}

// ---------------------------------------------------------------------------
// ComposingLayer — IME Composing 下划线
// ---------------------------------------------------------------------------
class ComposingLayer implements EditorLayer {
  final Color underlineColor;
  final double underlineWidth;

  ComposingLayer({
    this.underlineColor = const Color(0xFF4FC3F7),
    this.underlineWidth = 2.0,
  });

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final composing = document.composing;
    if (composing.isCollapsed) return;

    final startPos = document.offsetToPosition(composing.start);
    final endPos = document.offsetToPosition(composing.end);

    final line = layout.lineIndex;
    if (line < startPos.line || line > endPos.line) return;

    final colStart = line == startPos.line ? startPos.col : 0;
    final colEnd = line == endPos.line ? endPos.col : layout.text.length;

    final boxes = layout.painter.getBoxesForSelection(
      TextSelection(baseOffset: colStart, extentOffset: colEnd),
    );

    final paint = Paint()
      ..color = underlineColor
      ..strokeWidth = underlineWidth
      ..style = PaintingStyle.stroke;

    for (final box in boxes) {
      final rect = box.toRect().translate(0, layout.top);
      final y = rect.bottom - underlineWidth / 2;
      canvas.drawLine(
        Offset(rect.left, y),
        Offset(rect.right, y),
        paint,
      );
    }
  }
}

// ---------------------------------------------------------------------------
// RuntimeLayer — 执行行高亮 / 变量访问高亮
// ---------------------------------------------------------------------------
class RuntimeLayer implements EditorLayer {
  final int? currentLine; // 1-based, null = 无
  final List<({String name, String accessType})> accessedVars;
  final Color executionLineColor;
  final Color readVarColor;
  final Color writeVarColor;

  RuntimeLayer({
    this.currentLine,
    this.accessedVars = const [],
    this.executionLineColor = const Color(0x4D2196F3),
    this.readVarColor = const Color(0x261E88E5),
    this.writeVarColor = const Color(0x40FF6D00),
  });

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final line = layout.lineIndex + 1;

    // 执行行背景
    if (currentLine != null && line == currentLine) {
      canvas.drawRect(
        Rect.fromLTWH(0, layout.top, viewport.width, layout.height),
        Paint()..color = executionLineColor,
      );
    }

    // 变量访问高亮
    if (accessedVars.isEmpty) return;
    for (final varInfo in accessedVars) {
      final color = varInfo.accessType == 'Write' ? writeVarColor : readVarColor;
      _highlightWord(canvas, layout, varInfo.name, color);
    }
  }

  void _highlightWord(Canvas canvas, LineLayout layout, String word, Color color) {
    final text = layout.text;
    int start = 0;
    while (true) {
      final idx = text.indexOf(word, start);
      if (idx < 0) break;
      // 简单词边界检查
      final before = idx > 0 ? text[idx - 1] : ' ';
      final after = idx + word.length < text.length ? text[idx + word.length] : ' ';
      bool isWordChar(String c) => RegExp(r'[a-zA-Z0-9_]').hasMatch(c);
      if (!isWordChar(before) && !isWordChar(after)) {
        final boxes = layout.painter.getBoxesForSelection(
          TextSelection(baseOffset: idx, extentOffset: idx + word.length),
        );
        for (final box in boxes) {
          final rect = box.toRect().translate(0, layout.top);
          canvas.drawRect(rect, Paint()..color = color);
        }
      }
      start = idx + word.length;
    }
  }
}

// ---------------------------------------------------------------------------
// TutorialLayer — 教程行高亮
// ---------------------------------------------------------------------------
class TutorialLayer implements EditorLayer {
  final Set<int> tutorialLines; // 1-based
  final Color highlightColor;

  TutorialLayer({
    this.tutorialLines = const {},
    this.highlightColor = const Color(0x1AFFC107),
  });

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final line = layout.lineIndex + 1;
    if (tutorialLines.contains(line)) {
      canvas.drawRect(
        Rect.fromLTWH(0, layout.top, viewport.width, layout.height),
        Paint()..color = highlightColor,
      );
    }
  }
}
