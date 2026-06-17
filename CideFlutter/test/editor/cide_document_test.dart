import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/editor/cide_document.dart';

void main() {
  group('CideDocument', () {
    test('setText updates text and line offsets', () {
      final doc = CideDocument();
      doc.setText('line1\nline2\nline3');

      expect(doc.text, 'line1\nline2\nline3');
      expect(doc.lineCount, 3);
      expect(doc.lineText(0), 'line1');
      expect(doc.lineText(1), 'line2');
      expect(doc.lineText(2), 'line3');
    });

    test('applyEdit inserts text', () {
      final doc = CideDocument();
      doc.setText('hello world');

      doc.applyEdit(const EditOp(startOffset: 5, oldText: '', newText: ','));

      expect(doc.text, 'hello, world');
      expect(doc.undoStack.length, 1);
    });

    test('applyEdit deletes text', () {
      final doc = CideDocument();
      doc.setText('hello world');

      doc.applyEdit(const EditOp(startOffset: 5, oldText: ' world', newText: ''));

      expect(doc.text, 'hello');
    });

    test('undo restores previous text', () {
      final doc = CideDocument();
      doc.setText('abc');
      doc.applyEdit(const EditOp(startOffset: 3, oldText: '', newText: 'd'));

      expect(doc.text, 'abcd');

      doc.undo();

      expect(doc.text, 'abc');
      expect(doc.redoStack.length, 1);
    });

    test('redo re-applies undone edit', () {
      final doc = CideDocument();
      doc.setText('abc');
      doc.applyEdit(const EditOp(startOffset: 3, oldText: '', newText: 'd'));
      doc.undo();

      doc.redo();

      expect(doc.text, 'abcd');
    });

    test('offsetToLine returns correct line', () {
      final doc = CideDocument();
      doc.setText('a\nbb\nccc');

      expect(doc.offsetToLine(0), 0);
      expect(doc.offsetToLine(2), 1);
      expect(doc.offsetToLine(6), 2);
    });

    test('offsetToPosition and positionToOffset are inverse', () {
      final doc = CideDocument();
      doc.setText('hello\nworld');

      const pos = DocPosition(line: 1, col: 3);
      final offset = doc.positionToOffset(pos);
      expect(offset, 9);
      expect(doc.offsetToPosition(offset), pos);
    });

    test('positionToOffset clamps out of range', () {
      final doc = CideDocument();
      doc.setText('hi');

      expect(doc.positionToOffset(const DocPosition(line: 0, col: 100)), 2);
      expect(doc.positionToOffset(const DocPosition(line: 99, col: 0)), 2);
    });

    test('updateSelection changes selection', () {
      final doc = CideDocument();
      doc.setText('abc');
      doc.updateSelection(
        const DocSelection(
          base: DocPosition(line: 0, col: 0),
          extent: DocPosition(line: 0, col: 2),
        ),
      );

      expect(doc.selection.start, const DocPosition(line: 0, col: 0));
      expect(doc.selection.end, const DocPosition(line: 0, col: 2));
      expect(doc.selection.isCollapsed, isFalse);
    });

    test('computeDiff finds single insertion', () {
      final op = CideDocument.computeDiff('abc', 'aXbc');

      expect(op, isNotNull);
      expect(op!.startOffset, 1);
      expect(op.oldText, '');
      expect(op.newText, 'X');
    });

    test('computeDiff finds single deletion', () {
      final op = CideDocument.computeDiff('aXbc', 'abc');

      expect(op, isNotNull);
      expect(op!.startOffset, 1);
      expect(op.oldText, 'X');
      expect(op.newText, '');
    });

    test('computeDiff returns null when texts are equal', () {
      final op = CideDocument.computeDiff('same', 'same');

      expect(op, isNull);
    });

    test('lineStartOffset and lineEndOffset handle boundaries', () {
      final doc = CideDocument();
      doc.setText('a\nbb');

      expect(doc.lineStartOffset(-1), 0);
      expect(doc.lineStartOffset(0), 0);
      expect(doc.lineStartOffset(1), 2);
      expect(doc.lineEndOffset(0), 1);
      expect(doc.lineEndOffset(1), 4);
    });
  });
}
