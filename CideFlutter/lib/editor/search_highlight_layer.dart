import 'package:flutter/material.dart';
import 'cide_document.dart';
import 'editor_layers.dart';
import 'find_replace_controller.dart';

/// ---------------------------------------------------------------------------
/// SearchHighlightLayer — 搜索匹配高亮图层
/// ---------------------------------------------------------------------------
/// 为当前匹配项显示琥珀色背景，其他匹配项显示淡琥珀色边框。
/// ---------------------------------------------------------------------------

class SearchHighlightLayer implements EditorLayer {
  final List<SearchMatch> matches;
  final int currentMatchIndex;

  SearchHighlightLayer({
    this.matches = const [],
    this.currentMatchIndex = -1,
  });

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    if (matches.isEmpty) return;

    final line = layout.lineIndex;
    final lineStart = document.lineStartOffset(line);
    final lineEnd = document.lineEndOffset(line);

    for (int i = 0; i < matches.length; i++) {
      final match = matches[i];
      if (match.end <= lineStart || match.start >= lineEnd) continue;

      final start = (match.start - lineStart).clamp(0, layout.text.length);
      final end = (match.end - lineStart).clamp(start, layout.text.length);
      if (start >= end) continue;

      final isCurrent = i == currentMatchIndex;
      final boxes = layout.painter.getBoxesForSelection(
        TextSelection(baseOffset: start, extentOffset: end),
      );

      for (final box in boxes) {
        final rect = box.toRect().translate(0, layout.top);
        if (isCurrent) {
          canvas.drawRect(
            rect,
            Paint()..color = const Color(0xFFFFC107).withValues(alpha: 0.4),
          );
        } else {
          canvas.drawRect(
            Rect.fromLTRB(rect.left, rect.bottom - 2, rect.right, rect.bottom),
            Paint()..color = const Color(0xFFFFC107).withValues(alpha: 0.3),
          );
        }
      }
    }
  }
}
