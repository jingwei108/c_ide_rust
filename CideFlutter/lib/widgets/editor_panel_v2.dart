import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:re_highlight/languages/c.dart';
import 'package:re_highlight/re_highlight.dart';
import 'package:re_highlight/styles/atom-one-dark.dart';
import 'package:re_highlight/styles/atom-one-light.dart';
import '../editor/editor.dart';
import '../editor/gutter/gutter_view.dart';
import '../editor/gutter/gutter_context.dart';
import '../editor/gutter/line_number_column.dart';
import '../editor/gutter/heatmap_column.dart';
import '../editor/find_replace_controller.dart';
import '../editor/find_replace_overlay.dart';
import '../editor/search_highlight_layer.dart';
import '../providers/ide_provider.dart';
import '../providers/theme_provider.dart';
import '../providers/unified_provider.dart';
import 'package:cide/src/rust/unified/types.dart' as rust_unified;

/// ---------------------------------------------------------------------------
/// EditorPanelV2 — CideEditor 驱动的编辑器面板（re_editor 替代方案）
/// ---------------------------------------------------------------------------
/// 公共 API 与 EditorPanel 完全兼容，可直接替换。
/// ---------------------------------------------------------------------------

class EditorPanelV2 extends ConsumerStatefulWidget {
  final VoidCallback? onTap;
  final VoidCallback? onBlankTap;
  final VoidCallback? onDismissKeyboard;

  const EditorPanelV2({
    super.key,
    this.onTap,
    this.onBlankTap,
    this.onDismissKeyboard,
  });

  @override
  ConsumerState<EditorPanelV2> createState() => EditorPanelV2State();
}

class EditorPanelV2State extends ConsumerState<EditorPanelV2> {
  final GlobalKey<CideEditorState> _editorKey = GlobalKey();
  late final CideDocument _document;
  late final Highlight _highlight;
  late final AutocompleteController _autocompleteController;
  late final FindReplaceController _findReplaceController;
  OverlayEntry? _contextMenuOverlay;
  OverlayEntry? _autocompleteOverlay;
  OverlayEntry? _findReplaceOverlay;

  // 运行时高亮状态
  int _currentHighlightLine = 0;
  List<rust_unified.AccessedVar> _currentAccessedVars = [];
  Set<int> _currentTutorialLines = {};

  // 标记 document 是否刚被本地编辑（IME / 自绘键盘），用于屏蔽 build 中的外部 source 回写
  bool _documentDirty = false;

  @override
  void initState() {
    super.initState();
    final source = ref.read(ideProvider).source;
    _document = CideDocument();
    _document.setText(source);
    _document.addListener(_onDocumentChanged);

    _highlight = Highlight();
    _highlight.registerLanguage('c', langC);

    _autocompleteController = AutocompleteController();
    _autocompleteController.addListener(_onAutocompleteChanged);

    _findReplaceController = FindReplaceController();
    _findReplaceController.addListener(_onFindReplaceChanged);
  }

  @override
  void dispose() {
    _hideContextMenu();
    _hideAutocomplete();
    _hideFindReplace();
    _autocompleteController.removeListener(_onAutocompleteChanged);
    _findReplaceController.removeListener(_onFindReplaceChanged);
    _document.removeListener(_onDocumentChanged);
    super.dispose();
  }

  void _onDocumentChanged() {
    _documentDirty = true;
    // 同步到 ideProvider（延迟，避免 widget tree building 阶段修改 provider）
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _documentDirty = false;
      if (mounted) {
        ref.read(ideProvider.notifier).updateSource(_document.text);
      }
    });
    // 触发自动补全检测
    _updateAutocomplete();
  }

  // ---------------------------------------------------------------------------
  // 自动补全
  // ---------------------------------------------------------------------------
  void _updateAutocomplete() {
    final sel = _document.selection.base;
    final offset = _document.positionToOffset(sel);
    final textBefore = _document.text.substring(0, offset);
    _autocompleteController.update(textBefore);
  }

  void _onAutocompleteChanged() {
    if (_autocompleteController.visible) {
      _showAutocomplete();
    } else {
      _hideAutocomplete();
    }
  }

  void _showAutocomplete() {
    _hideAutocomplete();
    final position = _calculateCursorGlobalPosition();
    if (position == null) return;

    final isDark = ref.read(themeProvider) == ThemeMode.dark;

    _autocompleteOverlay = OverlayEntry(
      builder: (context) => Stack(
        children: [
          // 点击空白处关闭
          Positioned.fill(
            child: GestureDetector(
              onTap: _hideAutocomplete,
              child: Container(color: Colors.transparent),
            ),
          ),
          // 候选列表
          Positioned(
            left: position.dx,
            top: position.dy + 21.0, // 光标下方
            child: AutocompleteOverlay(
              controller: _autocompleteController,
              onDismiss: _hideAutocomplete,
              onSelected: _applyAutocomplete,
              isDark: isDark,
            ),
          ),
        ],
      ),
    );
    Overlay.of(context).insert(_autocompleteOverlay!);
  }

  void _hideAutocomplete() {
    _autocompleteOverlay?.remove();
    _autocompleteOverlay = null;
    _autocompleteController.hide();
  }

  // ---------------------------------------------------------------------------
  // 查找替换
  // ---------------------------------------------------------------------------
  void _onFindReplaceChanged() {
    if (_findReplaceController.visible) {
      _showFindReplace();
    } else {
      _hideFindReplace();
    }
  }

  void _showFindReplace() {
    _hideFindReplace();
    _findReplaceOverlay = OverlayEntry(
      builder: (context) => Positioned(
        top: 48,
        right: 16,
        child: FindReplaceOverlay(
          controller: _findReplaceController,
          onFindNext: _onFindNext,
          onFindPrevious: _onFindPrevious,
          onReplace: _onReplace,
          onReplaceAll: _onReplaceAll,
          onClose: _hideFindReplace,
          isDark: ref.read(themeProvider) == ThemeMode.dark,
        ),
      ),
    );
    Overlay.of(context).insert(_findReplaceOverlay!);
  }

  void _hideFindReplace() {
    _findReplaceOverlay?.remove();
    _findReplaceOverlay = null;
    _findReplaceController.hide();
  }

  void _onFindNext() {
    _findReplaceController.search(_document.text);
    _findReplaceController.nextMatch();
    _scrollToMatch();
  }

  void _onFindPrevious() {
    _findReplaceController.search(_document.text);
    _findReplaceController.previousMatch();
    _scrollToMatch();
  }

  void _scrollToMatch() {
    final match = _findReplaceController.currentMatch;
    if (match == null) return;
    final pos = _document.offsetToPosition(match.start);
    _editorKey.currentState?.scrollToLine(pos.line + 1);
    _document.updateSelection(
      DocSelection(base: pos, extent: pos),
    );
    _editorKey.currentState?.syncToProxy();
  }

  void _onReplace() {
    final match = _findReplaceController.currentMatch;
    if (match == null) return;
    _document.applyEdit(EditOp(
      startOffset: match.start,
      oldText: _document.text.substring(match.start, match.end),
      newText: _findReplaceController.replacement,
    ));
    _findReplaceController.search(_document.text);
  }

  void _onReplaceAll() {
    final query = _findReplaceController.query;
    final replacement = _findReplaceController.replacement;
    if (query.isEmpty) return;

    // 从后往前替换，避免 offset 漂移
    final matches = List<SearchMatch>.from(_findReplaceController.matches);
    matches.sort((a, b) => b.start.compareTo(a.start));

    for (final match in matches) {
      _document.applyEdit(EditOp(
        startOffset: match.start,
        oldText: _document.text.substring(match.start, match.end),
        newText: replacement,
      ));
    }
    _findReplaceController.search(_document.text);
  }

  void showFindReplace() => _findReplaceController.show();

  void _applyAutocomplete(AutocompleteCandidate candidate) {
    final sel = _document.selection.base;
    final cursorOffset = _document.positionToOffset(sel);
    final textBefore = _document.text.substring(0, cursorOffset);
    final prefix = AutocompleteController.extractPrefix(textBefore);
    final prefixOffset = cursorOffset - prefix.length;

    _document.applyEdit(EditOp(
      startOffset: prefixOffset,
      oldText: prefix,
      newText: candidate.word,
    ));

    // 移动光标到补全词后
    final newOffset = prefixOffset + candidate.word.length;
    final newPos = _document.offsetToPosition(newOffset);
    _document.updateSelection(
      DocSelection(base: newPos, extent: newPos),
    );
    _editorKey.currentState?.syncToProxy();
  }

  Offset? _calculateCursorGlobalPosition() {
    final context = _editorKey.currentContext;
    if (context == null) return null;
    final renderBox = context.findRenderObject() as RenderBox?;
    if (renderBox == null) return null;

    final sel = _document.selection.base;
    final line = sel.line;
    final col = sel.col;
    final lineHeight = 21.0;

    final lineText = _document.lineText(line);
    final safeCol = col.clamp(0, lineText.length);
    final textPainter = TextPainter(
      text: TextSpan(
        text: lineText.substring(0, safeCol),
        style: const TextStyle(fontSize: 14, height: 1.5, fontFamily: 'Consolas'),
      ),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout();

    // 编辑器内部坐标（相对于 EditableText/CustomPaint）
    final localOffset = Offset(textPainter.width, line * lineHeight);
    // 转换为全局坐标
    return renderBox.localToGlobal(localOffset);
  }

  // ---------------------------------------------------------------------------
  // 公共 API（与 EditorPanelState 兼容）
  // ---------------------------------------------------------------------------
  int getCurrentLine() => _editorKey.currentState?.getCurrentLine() ?? 0;

  void insertText(String text) => _editorKey.currentState?.insertText(text);

  void insertPair(String open, String close) =>
      _editorKey.currentState?.insertPair(open, close);

  void undo() => _editorKey.currentState?.undo();

  void redo() => _editorKey.currentState?.redo();

  void moveCursor(int offset) => _editorKey.currentState?.moveCursor(offset);

  void scrollToLine(int line) => _editorKey.currentState?.scrollToLine(line);

  void setText(String text) {
    if (_document.text == text) return;
    _document.setText(text);
  }

  FocusNode get focusNode =>
      _editorKey.currentState?.focusNode ?? FocusNode();

  void setReadOnly(bool value) => _editorKey.currentState?.setReadOnly(value);

  void showSystemKeyboard() => _editorKey.currentState?.showSystemKeyboard();

  void showCustomKeyboard() => _editorKey.currentState?.showCustomKeyboard();

  void backspace() => _editorKey.currentState?.backspace();

  void insertNewline() => _editorKey.currentState?.insertNewline();

  // ---------------------------------------------------------------------------
  // VS 风格 Enter：自动补分号
  // ---------------------------------------------------------------------------
  void _tryAppendSemicolon(int lineIndex, String lineText) {
    final trimmed = lineText.trimRight();
    if (trimmed.isEmpty) return;
    if (trimmed.endsWith(';') ||
        trimmed.endsWith('{') ||
        trimmed.endsWith('}')) {
      return;
    }
    if (trimmed.startsWith('//') ||
        trimmed.startsWith('/*') ||
        trimmed.startsWith('*')) {
      return;
    }
    if (trimmed.startsWith('#')) return;
    final needsSemicolon = RegExp(
      r'^(\s*(int|char|float|double|void|struct|enum|typedef|return|break|continue|printf|scanf|malloc|free|memcpy|memset|strlen|strcpy|strcmp|atoi|rand|srand|exit|getchar|putchar|fprintf|realloc|qsort)\b|.*[a-zA-Z_]\w*\s*=|.*\)\s*$)',
    );
    if (needsSemicolon.hasMatch(trimmed)) {
      final newLine = '$trimmed;';
      final startOffset = _document.lineStartOffset(lineIndex);
      _document.applyEdit(EditOp(
        startOffset: startOffset,
        oldText: lineText,
        newText: newLine,
      ));
    }
  }

  // ---------------------------------------------------------------------------
  // 长按菜单
  // ---------------------------------------------------------------------------
  Timer? _longPressTimer;
  Offset? _longPressStart;
  Offset? _swipeStart;
  static const _longPressDuration = Duration(milliseconds: 600);
  static const _longPressMoveThreshold = 15.0;

  void _startLongPress(Offset position) {
    _cancelLongPress();
    _longPressStart = position;
    _longPressTimer = Timer(_longPressDuration, () {
      _longPressTimer = null;
      _longPressStart = null;
      _showContextMenu(position);
    });
  }

  void _checkLongPressMove(Offset position) {
    if (_longPressStart == null) return;
    if ((position - _longPressStart!).distance > _longPressMoveThreshold) {
      _cancelLongPress();
    }
  }

  void _cancelLongPress() {
    _longPressTimer?.cancel();
    _longPressTimer = null;
    _longPressStart = null;
  }

  void _hideContextMenu() {
    _contextMenuOverlay?.remove();
    _contextMenuOverlay = null;
  }

  void _showContextMenu(Offset position) {
    _hideContextMenu();

    final hasSelection = !_document.selection.isCollapsed;
    final screenSize = MediaQuery.of(context).size;

    final itemCount = (hasSelection ? 1 : 0) + 3;
    const estimatedItemWidth = 64.0;
    const estimatedDividerWidth = 1.0;
    const horizontalPadding = 32.0;
    final estimatedWidth =
        itemCount * estimatedItemWidth + (itemCount - 1) * estimatedDividerWidth + horizontalPadding;
    final needsWrap = estimatedWidth > screenSize.width - 32;

    final overlay = Overlay.of(context);
    _contextMenuOverlay = OverlayEntry(
      builder: (context) => _ContextMenuBar(
        position: position,
        needsWrap: needsWrap,
        hasSelection: hasSelection,
        onCopy: hasSelection
            ? () {
                _editorKey.currentState?.copy();
                _hideContextMenu();
              }
            : null,
        onPaste: () async {
          await _editorKey.currentState?.paste();
          _hideContextMenu();
        },
        onSelectAll: () {
          _editorKey.currentState?.selectAll();
          _hideContextMenu();
        },
        onDismiss: _hideContextMenu,
      ),
    );
    overlay.insert(_contextMenuOverlay!);
  }

  // ---------------------------------------------------------------------------
  // 构建
  // ---------------------------------------------------------------------------
  @override
  Widget build(BuildContext context) {
    final state = ref.watch(ideProvider);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;
    final unifiedState = ref.watch(unifiedProvider);

    // 同步外部 source 变更（如文件切换、修复应用）
    // 当 document 正被本地编辑时（IME / 自绘键盘），禁止把滞后的 state.source 回写，
    // 否则会和输入发生 race condition，导致文本错乱或光标跳回。
    if (!_documentDirty && _document.text != state.source) {
      _document.setText(state.source);
    }

    // 更新运行时高亮状态
    int newHighlightLine = 0;
    List<rust_unified.AccessedVar> newAccessedVars = [];
    if (unifiedState.currentStep >= 0 &&
        unifiedState.currentStep < unifiedState.frameCache.length) {
      final payload = unifiedState.frameCache[unifiedState.currentStep];
      newHighlightLine = payload.codeLine;
      newAccessedVars = payload.accessedVars;
    }

    final newTutorialLines = state.activeTutorial?.focusLines.toSet() ?? <int>{};

    final hasHighlightChanged =
        newHighlightLine != _currentHighlightLine ||
        !_accessedVarsEqual(newAccessedVars, _currentAccessedVars) ||
        !_tutorialLinesEqual(newTutorialLines, _currentTutorialLines);

    if (hasHighlightChanged) {
      _currentHighlightLine = newHighlightLine;
      _currentAccessedVars = List.from(newAccessedVars);
      _currentTutorialLines = newTutorialLines;
    }

    final editorBackground =
        isDark ? const Color(0xff282c34) : const Color(0xfffafafa);
    final editorTextColor =
        isDark ? const Color(0xffabb2bf) : const Color(0xff383a42);
    final separatorColor =
        isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5);
    final cursorColor = isDark ? Colors.white : Colors.black;
    final cursorLineColor = state.isStepMode || state.isRunning
        ? Colors.blueAccent.withValues(alpha: 0.3)
        : null;

    final textStyle = TextStyle(
      fontSize: 14,
      height: 1.5,
      fontFamily: 'Consolas',
      fontFamilyFallback: const ['monospace'],
      color: editorTextColor,
    );

    final theme = isDark ? atomOneDarkTheme : atomOneLightTheme;

    // 构建图层
    final layers = <EditorLayer>[
      SyntaxHighlightLayer(
        baseStyle: textStyle,
        highlight: _highlight,
        theme: theme,
      ),
      SelectionLayer(cursorColor: cursorColor),
      ComposingLayer(),
      if (cursorLineColor != null)
        _CursorLineLayer(color: cursorLineColor),
      RuntimeLayer(
        currentLine: _currentHighlightLine,
        accessedVars: _currentAccessedVars
            .map((a) => (name: a.name, accessType: a.accessType))
            .toList(),
      ),
      TutorialLayer(tutorialLines: _currentTutorialLines),
      DiagnosticLayer(
        diagnostics: state.diagnostics.map((d) {
          // Rust 后端输出列号为 1-based，转换为 0-based
          final startCol = (d.replaceStartColumn > 0
                  ? d.replaceStartColumn
                  : d.column) -
              1;
          final endCol = d.replaceEndColumn > 0
              ? d.replaceEndColumn - 1
              : startCol + 1;
          return DiagnosticInfo(
            line: d.line,
            severity: d.severity,
            startCol: startCol.clamp(0, 9999),
            endCol: endCol.clamp(startCol, 9999),
          );
        }).toList(),
      ),
      SearchHighlightLayer(
        matches: _findReplaceController.matches,
        currentMatchIndex: _findReplaceController.currentMatchIndex,
      ),
    ];

    return Container(
      decoration: BoxDecoration(
        color: editorBackground,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          // Gutter（插件化）
          GutterView(
            columns: [
              HeatmapColumn(),
              LineNumberColumn(),
            ],
            context: GutterContext(
              currentLine: getCurrentLine(),
              currentDebugLine: state.currentLine,
              isStepMode: state.isStepMode,
              breakpoints: state.breakpoints,
              diagMap: {
                for (final d in state.diagnostics)
                  d.line: d.severity
              },
              accessedVars: _currentAccessedVars,
              heatmap: unifiedState.heatmap,
              isDark: isDark,
            ),
            scrollOffset: _editorKey.currentState?.scrollController.offset ?? 0.0,
            viewportHeight: MediaQuery.of(context).size.height,
            lineHeight: 21.0,
            lineCount: _document.lineCount,
            onTapLine: () {},
          ),
          // 分隔线
          Container(width: 1, color: separatorColor),
          // 编辑器主体
          Expanded(
            child: Listener(
              onPointerDown: (event) {
                _swipeStart = event.position;
                _startLongPress(event.position);
              },
              onPointerMove: (event) => _checkLongPressMove(event.position),
              onPointerUp: (event) {
                final wasShortPress = _longPressTimer != null;
                _cancelLongPress();

                // 检测上下滑动手势收起键盘
                if (_swipeStart != null && widget.onDismissKeyboard != null) {
                  final dx = event.position.dx - _swipeStart!.dx;
                  final dy = event.position.dy - _swipeStart!.dy;
                  if (dy.abs() > 100 && dy.abs() > dx.abs() * 1.5) {
                    widget.onDismissKeyboard!();
                    _swipeStart = null;
                    return;
                  }
                }
                _swipeStart = null;

                // 短按：判断空白处
                if (wasShortPress) {
                  WidgetsBinding.instance.addPostFrameCallback((_) {
                    if (!mounted) return;
                    final sel = _document.selection;
                    final line = sel.base.line;
                    if (line >= 0 && line < _document.lineCount) {
                      final lineText = _document.lineText(line);
                      final col = sel.base.col;
                      final isBlank = lineText.trim().isEmpty ||
                          col >= lineText.length ||
                          col >= lineText.trimRight().length;
                      if (isBlank) {
                        widget.onBlankTap?.call();
                      } else {
                        widget.onTap?.call();
                      }
                    } else {
                      widget.onBlankTap?.call();
                    }
                  });
                }
              },
              behavior: HitTestBehavior.translucent,
              child: CideEditor(
                key: _editorKey,
                document: _document,
                style: textStyle,
                layers: layers,
                onTryAppendSemicolon: _tryAppendSemicolon,
              ),
            ),
          ),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Gutter
  // ---------------------------------------------------------------------------

  // ---------------------------------------------------------------------------
  // 工具方法
  // ---------------------------------------------------------------------------
  bool _accessedVarsEqual(
    List<rust_unified.AccessedVar> a,
    List<rust_unified.AccessedVar> b,
  ) {
    if (a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) {
      if (a[i].name != b[i].name || a[i].accessType != b[i].accessType) {
        return false;
      }
    }
    return true;
  }

  bool _tutorialLinesEqual(Set<int> a, Set<int> b) {
    if (a.length != b.length) return false;
    return a.containsAll(b);
  }

}

// ---------------------------------------------------------------------------
// _CursorLineLayer — 光标行背景
// ---------------------------------------------------------------------------
class _CursorLineLayer implements EditorLayer {
  final Color color;

  _CursorLineLayer({required this.color});

  @override
  void paint(Canvas canvas, LineLayout layout, CideDocument document, Rect viewport) {
    final sel = document.selection;
    if (sel.isCollapsed && sel.base.line == layout.lineIndex) {
      canvas.drawRect(
        Rect.fromLTWH(0, layout.top, viewport.width, layout.height),
        Paint()..color = color,
      );
    }
  }
}

// ---------------------------------------------------------------------------
// 长按上下文菜单（复用 EditorPanel 的设计）
// ---------------------------------------------------------------------------
class _ContextMenuBar extends StatelessWidget {
  final Offset position;
  final bool needsWrap;
  final bool hasSelection;
  final VoidCallback? onCopy;
  final VoidCallback? onPaste;
  final VoidCallback? onSelectAll;
  final VoidCallback onDismiss;

  const _ContextMenuBar({
    required this.position,
    required this.needsWrap,
    required this.hasSelection,
    this.onCopy,
    this.onPaste,
    this.onSelectAll,
    required this.onDismiss,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final bgColor = isDark ? const Color(0xFF3A3A3C) : const Color(0xFFD1D1D6);
    final textColor = isDark ? Colors.white : Colors.black87;
    final dividerColor = isDark ? Colors.white24 : Colors.black26;
    final screenSize = MediaQuery.of(context).size;
    final safeTop = MediaQuery.of(context).padding.top;
    final safeBottom = MediaQuery.of(context).padding.bottom;

    const singleLineHeight = 44.0;
    const twoLineHeight = 88.0;
    final menuHeight = needsWrap ? twoLineHeight : singleLineHeight;

    double top = position.dy - menuHeight - 8;
    if (top < safeTop + 8) {
      top = position.dy + 8;
    }
    if (top + menuHeight > screenSize.height - safeBottom - 8) {
      top = position.dy - menuHeight - 8;
    }

    final itemCount = (hasSelection ? 1 : 0) + 3;
    const estimatedItemWidth = 64.0;
    const estimatedDividerWidth = 1.0;
    const horizontalPadding = 32.0;
    final estimatedWidth = needsWrap
        ? ((itemCount + 1) ~/ 2) * estimatedItemWidth +
            (((itemCount + 1) ~/ 2) - 1) * estimatedDividerWidth +
            horizontalPadding
        : itemCount * estimatedItemWidth +
            (itemCount - 1) * estimatedDividerWidth +
            horizontalPadding;

    double left = position.dx - estimatedWidth / 2;
    if (left < 16) left = 16;
    if (left + estimatedWidth > screenSize.width - 16) {
      left = screenSize.width - estimatedWidth - 16;
    }
    if (left < 16) left = 16;

    List<Widget> buildItems() {
      final items = <Widget>[];
      if (onCopy != null) {
        items.add(_MenuButton(text: '复制', onTap: onCopy!, textColor: textColor));
        items.add(_MenuDivider(color: dividerColor));
      }
      if (onPaste != null) {
        items.add(_MenuButton(text: '粘贴', onTap: onPaste!, textColor: textColor));
        items.add(_MenuDivider(color: dividerColor));
      }
      if (onSelectAll != null) {
        items.add(_MenuButton(text: '全选', onTap: onSelectAll!, textColor: textColor));
        items.add(_MenuDivider(color: dividerColor));
      }
      items.add(_MenuButton(text: '取消', onTap: onDismiss, textColor: textColor));
      return items;
    }

    final items = buildItems();

    return Stack(
      children: [
        GestureDetector(
          onTap: onDismiss,
          child: Container(color: Colors.transparent),
        ),
        Positioned(
          left: left,
          top: top,
          child: Container(
            decoration: BoxDecoration(
              color: bgColor,
              borderRadius: BorderRadius.circular(12),
            ),
            child: IntrinsicWidth(
              child: needsWrap
                  ? Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Row(mainAxisSize: MainAxisSize.min, children: items.take(2).toList()),
                        Divider(height: 1, color: dividerColor),
                        Row(mainAxisSize: MainAxisSize.min, children: items.skip(2).toList()),
                      ],
                    )
                  : Row(mainAxisSize: MainAxisSize.min, children: items),
            ),
          ),
        ),
      ],
    );
  }
}

class _MenuButton extends StatelessWidget {
  final String text;
  final VoidCallback onTap;
  final Color textColor;

  const _MenuButton({
    required this.text,
    required this.onTap,
    required this.textColor,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Text(
          text,
          style: TextStyle(fontSize: 14, color: textColor),
        ),
      ),
    );
  }
}

class _MenuDivider extends StatelessWidget {
  final Color color;

  const _MenuDivider({required this.color});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 1,
      height: 24,
      margin: const EdgeInsets.symmetric(vertical: 10),
      color: color,
    );
  }
}
