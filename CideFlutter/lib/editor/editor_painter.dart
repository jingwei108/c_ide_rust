import 'package:flutter/material.dart';
import 'cide_document.dart';
import 'editor_layers.dart';

/// ---------------------------------------------------------------------------
/// CideEditorPainter — 代码画布主绘制器
/// ---------------------------------------------------------------------------
/// 职责：
/// - 遍历所有可见行，为每行构建 TextPainter 布局
/// - 按顺序调用各 EditorLayer 的 paint 方法
/// - 只绘制视口内行，跳过不可见区域
/// ---------------------------------------------------------------------------

class _CachedPainter {
  final String text;
  final TextStyle style;
  final TextPainter painter;

  _CachedPainter({required this.text, required this.style, required this.painter});
}

class CideEditorPainter extends CustomPainter {
  final CideDocument document;
  final double scrollOffset;
  final double viewportHeight;
  final double lineHeight;
  final TextStyle textStyle;
  final List<EditorLayer> layers;

  // 构造时快照，用于 shouldRepaint 比较（避免 document 被修改后比较的是同一对象）
  final String _text;
  final DocSelection _selection;
  final TextRange _composing;

  // 静态缓存：同一进程内所有编辑器实例共享，key 为行号。
  // 缓存条目在文本/样式变化时通过 _cacheKey 校验失效并重建。
  static final Map<int, _CachedPainter> _textPainterCache = {};

  CideEditorPainter({
    required this.document,
    required this.scrollOffset,
    required this.viewportHeight,
    required this.lineHeight,
    required this.textStyle,
    required this.layers,
  })  : _text = document.text,
        _selection = document.selection,
        _composing = document.composing;

  TextPainter _getTextPainter(int line, String text) {
    final cached = _textPainterCache[line];
    if (cached != null && cached.text == text && cached.style == textStyle) {
      return cached.painter;
    }
    final painter = TextPainter(
      text: TextSpan(text: text, style: textStyle),
      textDirection: TextDirection.ltr,
    );
    painter.layout();
    _textPainterCache[line] = _CachedPainter(text: text, style: textStyle, painter: painter);
    return painter;
  }

  @override
  void paint(Canvas canvas, Size size) {
    final viewport = Rect.fromLTWH(0, scrollOffset, size.width, viewportHeight);

    // 计算可见行范围
    final firstVisibleLine = (scrollOffset / lineHeight).floor().clamp(0, document.lineCount - 1);
    final lastVisibleLine =
        ((scrollOffset + viewportHeight) / lineHeight).ceil().clamp(0, document.lineCount - 1);

    for (int line = firstVisibleLine; line <= lastVisibleLine; line++) {
      final text = _safeLineText(line);
      final top = line * lineHeight;

      // 跳过完全在视口外的行
      if (top + lineHeight < scrollOffset || top > scrollOffset + viewportHeight) {
        continue;
      }

      // 使用缓存的 TextPainter，仅在行文本或样式变化时重建。
      final textPainter = _getTextPainter(line, text);

      final layout = LineLayout(
        lineIndex: line,
        text: text,
        top: top,
        height: lineHeight,
        painter: textPainter,
      );

      // 按顺序绘制各图层
      for (final layer in layers) {
        layer.paint(canvas, layout, document, viewport);
      }
    }
  }

  String _safeLineText(int line) {
    if (line < 0 || line >= document.lineCount) return '';
    return document.lineText(line);
  }

  @override
  bool shouldRepaint(covariant CideEditorPainter old) {
    return old._text != _text ||
        old._selection != _selection ||
        old._composing != _composing ||
        old.scrollOffset != scrollOffset ||
        old.viewportHeight != viewportHeight ||
        old.lineHeight != lineHeight ||
        old.textStyle != textStyle ||
        old.layers.length != layers.length;
  }
}
