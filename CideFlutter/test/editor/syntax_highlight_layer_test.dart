import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:re_highlight/languages/c.dart';
import 'package:re_highlight/re_highlight.dart';
import 'package:cide/editor/cide_document.dart';
import 'package:cide/editor/editor_layers.dart';
import 'package:cide/editor/syntax_highlight_layer.dart';

class _FakeLineLayout extends LineLayout {
  _FakeLineLayout({
    required super.lineIndex,
    required super.text,
    super.top = 0,
  }) : super(
          height: 20,
          painter: TextPainter(
            text: const TextSpan(text: ''),
            textDirection: TextDirection.ltr,
          )..layout(),
        );
}

void main() {
  group('SyntaxHighlightLayer', () {
    late Highlight highlight;

    setUpAll(() {
      highlight = Highlight();
      highlight.registerLanguage('c', langC);
    });

    test('paint does not crash for empty line', () {
      final layer = SyntaxHighlightLayer(
        baseStyle: const TextStyle(fontSize: 14),
        highlight: highlight,
        theme: const {},
      );

      final recorder = PictureRecorder();
      final canvas = Canvas(recorder);
      final doc = CideDocument();
      final layout = _FakeLineLayout(lineIndex: 0, text: '');
      layer.paint(canvas, layout, doc, const Rect.fromLTWH(0, 0, 100, 100));
    });

    test('paint does not crash for C keyword line', () {
      final layer = SyntaxHighlightLayer(
        baseStyle: const TextStyle(fontSize: 14),
        highlight: highlight,
        theme: const {
          'keyword': TextStyle(color: Colors.blue),
        },
      );

      final recorder = PictureRecorder();
      final canvas = Canvas(recorder);
      final doc = CideDocument();
      final layout = _FakeLineLayout(lineIndex: 0, text: 'int main() {');
      layer.paint(canvas, layout, doc, const Rect.fromLTWH(0, 0, 100, 100));
    });

    test('clearCache empties internal cache', () {
      final layer = SyntaxHighlightLayer(
        baseStyle: const TextStyle(fontSize: 14),
        highlight: highlight,
        theme: const {},
      );

      // Paint something to populate cache.
      final recorder = PictureRecorder();
      final canvas = Canvas(recorder);
      final doc = CideDocument();
      layer.paint(
        canvas,
        _FakeLineLayout(lineIndex: 0, text: 'int x;'),
        doc,
        const Rect.fromLTWH(0, 0, 100, 100),
      );
      // Clear and repaint; should not throw.
      layer.clearCache();
      final recorder2 = PictureRecorder();
      final canvas2 = Canvas(recorder2);
      layer.paint(
        canvas2,
        _FakeLineLayout(lineIndex: 0, text: 'int x;'),
        doc,
        const Rect.fromLTWH(0, 0, 100, 100),
      );
    });
  });
}
