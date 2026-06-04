import 'package:flutter/material.dart';
import 'package:re_highlight/re_highlight.dart';
import 'cide_document.dart';
import 'editor_layers.dart';

/// ---------------------------------------------------------------------------
/// SyntaxHighlightLayer — C 语言语法高亮图层
/// ---------------------------------------------------------------------------
/// 使用 re_highlight 为每行生成带语法高亮的 TextSpan。
/// 对空行/纯文本行不做高亮（直接复用 baseStyle）。
/// ---------------------------------------------------------------------------

class SyntaxHighlightLayer implements EditorLayer {
  final TextStyle baseStyle;
  final Highlight highlight;
  final Map<String, TextStyle> theme;

  // 简单行级缓存
  final Map<String, TextSpan> _cache = {};

  SyntaxHighlightLayer({
    required this.baseStyle,
    required this.highlight,
    required this.theme,
  });

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final cached = _cache[layout.text];
    final TextPainter painter;

    if (cached != null) {
      painter = TextPainter(
        text: cached,
        textDirection: TextDirection.ltr,
      );
    } else {
      final span = _highlightLine(layout.text);
      _cache[layout.text] = span;
      painter = TextPainter(
        text: span,
        textDirection: TextDirection.ltr,
      );
    }

    painter.layout();
    painter.paint(canvas, Offset(0, layout.top));
  }

  TextSpan _highlightLine(String text) {
    if (text.isEmpty) {
      return TextSpan(text: '', style: baseStyle);
    }
    try {
      final result = highlight.highlight(code: text, language: 'c');
      final renderer = TextSpanRenderer(baseStyle, theme);
      result.render(renderer);
      return renderer.span ?? TextSpan(text: text, style: baseStyle);
    } catch (_) {
      return TextSpan(text: text, style: baseStyle);
    }
  }

  void clearCache() => _cache.clear();
}
