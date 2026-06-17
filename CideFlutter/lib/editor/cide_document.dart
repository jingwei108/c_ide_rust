import 'dart:math';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// ---------------------------------------------------------------------------
/// CideDocument — Cide 编辑器文档模型
/// ---------------------------------------------------------------------------
/// 职责：
/// - 持有纯文本与选区（行/列坐标）
/// - 维护换行符索引（O(log n) 行号查询）
/// - 提供增量 edit / undo / redo
/// - 向外通知变更（NotifyListener 模式）
/// ---------------------------------------------------------------------------

/// 单次编辑操作（可用于 apply / undo / redo）
class EditOp {
  final int startOffset;
  final String oldText;
  final String newText;

  const EditOp({
    required this.startOffset,
    required this.oldText,
    required this.newText,
  });

  EditOp inverse() => EditOp(
        startOffset: startOffset,
        oldText: newText,
        newText: oldText,
      );

  @override
  String toString() => 'EditOp($startOffset, "$oldText" -> "$newText")';
}

/// 行+列坐标
@immutable
class DocPosition {
  final int line; // 0-based
  final int col; // 0-based

  const DocPosition({required this.line, required this.col});

  @override
  bool operator ==(Object other) =>
      other is DocPosition && other.line == line && other.col == col;

  @override
  int get hashCode => Object.hash(line, col);

  @override
  String toString() => 'DocPosition($line, $col)';
}

/// 选区（行/列坐标）
@immutable
class DocSelection {
  final DocPosition base;
  final DocPosition extent;

  const DocSelection({
    required this.base,
    required this.extent,
  });

  bool get isCollapsed =>
      base.line == extent.line && base.col == extent.col;

  DocPosition get start =>
      (base.line < extent.line ||
              (base.line == extent.line && base.col <= extent.col))
          ? base
          : extent;

  DocPosition get end =>
      (base.line < extent.line ||
              (base.line == extent.line && base.col <= extent.col))
          ? extent
          : base;

  @override
  bool operator ==(Object other) =>
      other is DocSelection && other.base == base && other.extent == extent;

  @override
  int get hashCode => Object.hash(base, extent);

  @override
  String toString() => 'DocSelection($base -> $extent)';
}

class CideDocument extends ChangeNotifier {
  String _text = '';
  final List<int> _lineStartOffsets = [0];

  // Undo / Redo
  final List<EditOp> _undoStack = [];
  final List<EditOp> _redoStack = [];
  static const int _maxHistory = 200;

  // 选区
  DocSelection _selection = const DocSelection(
    base: DocPosition(line: 0, col: 0),
    extent: DocPosition(line: 0, col: 0),
  );

  // Composing range（全局 offset，来自 IME）
  TextRange _composing = TextRange.empty;

  // ---------------------------------------------------------------------------
  // 公开属性
  // ---------------------------------------------------------------------------
  String get text => _text;

  DocSelection get selection => _selection;

  TextRange get composing => _composing;

  int get lineCount => _lineStartOffsets.length;

  List<EditOp> get undoStack => List.unmodifiable(_undoStack);

  List<EditOp> get redoStack => List.unmodifiable(_redoStack);

  // ---------------------------------------------------------------------------
  // 文本操作
  // ---------------------------------------------------------------------------
  void setText(String text) {
    if (_text == text) return;
    _text = text;
    _rebuildLineOffsets();
    notifyListeners();
  }

  /// 批量设置文本 + 选区 + composing，只触发一次 notifyListeners
  void setTextSync(String text, DocSelection selection, TextRange composing) {
    if (_text != text) {
      _text = text;
      _rebuildLineOffsets();
    }
    _selection = selection;
    _composing = composing;
    notifyListeners();
  }

  /// 直接应用 EditOp（不记录历史，用于内部/undo）
  void _apply(EditOp op) {
    _text = _text.substring(0, op.startOffset) +
        op.newText +
        _text.substring(op.startOffset + op.oldText.length);
    // 全量重建行索引，避免增量重建在旧索引上计算 startLine 的时序 bug
    _rebuildLineOffsets();
  }

  /// 应用编辑并记录 Undo 历史
  void applyEdit(EditOp op) {
    _apply(op);
    _undoStack.add(op);
    if (_undoStack.length > _maxHistory) _undoStack.removeAt(0);
    _redoStack.clear();
    notifyListeners();
  }

  /// 批量应用编辑 + 选区 + composing，只触发一次 notifyListeners
  void applyEditSync(EditOp op, DocSelection selection, TextRange composing) {
    _apply(op);
    _undoStack.add(op);
    if (_undoStack.length > _maxHistory) _undoStack.removeAt(0);
    _redoStack.clear();
    _selection = selection;
    _composing = composing;
    notifyListeners();
  }

  void undo() {
    if (_undoStack.isEmpty) return;
    final op = _undoStack.removeLast().inverse();
    _apply(op);
    _redoStack.add(op.inverse());
    notifyListeners();
  }

  void redo() {
    if (_redoStack.isEmpty) return;
    final op = _redoStack.removeLast();
    _apply(op);
    _undoStack.add(op.inverse());
    notifyListeners();
  }

  void clearUndoStack() {
    _undoStack.clear();
    _redoStack.clear();
  }

  // ---------------------------------------------------------------------------
  // 选区 / Composing
  // ---------------------------------------------------------------------------
  void updateSelection(DocSelection sel) {
    _selection = sel;
    notifyListeners();
  }

  void updateComposing(TextRange range) {
    _composing = range;
    notifyListeners();
  }

  // ---------------------------------------------------------------------------
  // Offset <-> Line/Col 转换（O(log n)）
  // ---------------------------------------------------------------------------
  int offsetToLine(int offset) {
    int lo = 0;
    int hi = _lineStartOffsets.length - 1;
    while (lo < hi) {
      final mid = (lo + hi + 1) ~/ 2;
      if (_lineStartOffsets[mid] <= offset) {
        lo = mid;
      } else {
        hi = mid - 1;
      }
    }
    return lo;
  }

  int offsetToCol(int offset) {
    final line = offsetToLine(offset);
    return offset - _lineStartOffsets[line];
  }

  DocPosition offsetToPosition(int offset) {
    final line = offsetToLine(offset);
    return DocPosition(
      line: line,
      col: offset - _lineStartOffsets[line],
    );
  }

  int positionToOffset(DocPosition pos) {
    if (pos.line < 0) return 0;
    if (pos.line >= _lineStartOffsets.length) {
      return _text.length;
    }
    final lineStart = _lineStartOffsets[pos.line];
    final lineEnd = pos.line + 1 < _lineStartOffsets.length
        ? _lineStartOffsets[pos.line + 1] - 1
        : _text.length;
    // clamp 到 [lineStart, lineEnd]，允许 col 越界时回到行尾
    return (lineStart + pos.col).clamp(lineStart, lineEnd);
  }

  /// 某行的起始 offset（含换行符前的字符）
  int lineStartOffset(int line) {
    if (line < 0) return 0;
    if (line >= _lineStartOffsets.length) return _text.length;
    return _lineStartOffsets[line];
  }

  /// 某行的结束 offset（不含换行符）
  int lineEndOffset(int line) {
    if (line < 0) return 0;
    if (line + 1 >= _lineStartOffsets.length) return _text.length;
    return _lineStartOffsets[line + 1] - 1;
  }

  String lineText(int line) {
    final start = lineStartOffset(line);
    final end = lineEndOffset(line);
    return _text.substring(start, end);
  }

  // ---------------------------------------------------------------------------
  // 行偏移索引维护
  // ---------------------------------------------------------------------------
  void _rebuildLineOffsets() {
    _lineStartOffsets.clear();
    _lineStartOffsets.add(0);
    for (int i = 0; i < _text.length; i++) {
      if (_text.codeUnitAt(i) == 0x0A) {
        _lineStartOffsets.add(i + 1);
      }
    }
  }

  /// 从受影响的 offset 所在行开始局部重建，比全量重建更快

  // ---------------------------------------------------------------------------
  // Diff（用于 Proxy ↔ Document 同步）
  // ---------------------------------------------------------------------------
  static EditOp? computeDiff(String oldText, String newText) {
    int commonPrefix = 0;
    final minLen = min(oldText.length, newText.length);
    while (commonPrefix < minLen &&
        oldText.codeUnitAt(commonPrefix) == newText.codeUnitAt(commonPrefix)) {
      commonPrefix++;
    }

    int commonSuffix = 0;
    final maxSuffix = min(oldText.length - commonPrefix, newText.length - commonPrefix);
    while (commonSuffix < maxSuffix) {
      final oIdx = oldText.length - 1 - commonSuffix;
      final nIdx = newText.length - 1 - commonSuffix;
      if (oldText.codeUnitAt(oIdx) != newText.codeUnitAt(nIdx)) break;
      commonSuffix++;
    }

    final start = commonPrefix;
    final oldEnd = oldText.length - commonSuffix;
    final newEnd = newText.length - commonSuffix;

    if (start == oldEnd && start == newEnd) return null; // 无变化

    return EditOp(
      startOffset: start,
      oldText: oldText.substring(start, oldEnd),
      newText: newText.substring(start, newEnd),
    );
  }

  /// 从全局 offset 创建插入操作
  EditOp createInsertOp(int offset, String text) {
    return EditOp(startOffset: offset, oldText: '', newText: text);
  }

  /// 从全局 offset 创建删除操作
  EditOp createDeleteOp(int offset, int length) {
    return EditOp(
      startOffset: offset,
      oldText: _text.substring(offset, offset + length),
      newText: '',
    );
  }

  @override
  String toString() =>
      'CideDocument(lines=$lineCount, len=${_text.length})';
}
