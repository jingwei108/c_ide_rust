import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:re_editor/re_editor.dart';
import 'package:re_highlight/languages/c.dart';
import 'package:re_highlight/styles/atom-one-dark.dart';
import 'package:re_highlight/styles/atom-one-light.dart';
import '../models/ide_state.dart';
import '../providers/ide_provider.dart';
import '../providers/theme_provider.dart';
import '../providers/unified_provider.dart';
import 'package:cide/src/rust/unified/types.dart' as rust_unified;

class EditorPanel extends ConsumerStatefulWidget {
  final VoidCallback? onTap;
  final VoidCallback? onBlankTap;
  final VoidCallback? onDismissKeyboard;

  const EditorPanel({
    super.key,
    this.onTap,
    this.onBlankTap,
    this.onDismissKeyboard,
  });

  @override
  ConsumerState<EditorPanel> createState() => EditorPanelState();
}

class EditorPanelState extends ConsumerState<EditorPanel> {
  late CodeLineEditingController _controller;
  final _focusNode = FocusNode();
  final _codeEditorKey = GlobalKey();
  bool _readOnly = true;
  OverlayEntry? _contextMenuOverlay;

  /// 获取当前光标所在行号（1-based），无焦点时返回 0
  int getCurrentLine() {
    if (_controller.selection.baseIndex < 0) return 0;
    return _controller.selection.baseIndex + 1;
  }

  @override
  void initState() {
    super.initState();
    final source = ref.read(ideProvider).source;
    _controller = CodeLineEditingController(
      codeLines: source.codeLines,
      options: const CodeLineOptions(indentSize: 4),
      spanBuilder: _buildVariableHighlightSpan,
    );
    _lastLineCount = _controller.lineCount;
    // 延迟添加 listener，避免 re_editor 初始化 delegate 时立即触发通知，
    // 导致在 widget tree building 阶段修改 provider
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        _controller.addListener(_onChanged);
      }
    });
  }

  int _lastLineCount = 0;

  void _onChanged() {
    // 延迟更新 provider，避免 re_editor 初始化 delegate 时触发 notifyListeners，
    // 导致在 widget tree building 阶段修改 provider
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        ref.read(ideProvider.notifier).updateSource(_controller.text);
      }
    });

    // VS 风格 Enter 格式化：检测换行，对前一行补分号
    final currentLineCount = _controller.lineCount;
    if (currentLineCount > _lastLineCount && _lastLineCount > 0) {
      // 行数增加，说明发生了换行
      final lineIndex = _controller.selection.startIndex - 1;
      if (lineIndex >= 0 && lineIndex < _controller.lineCount) {
        _tryAppendSemicolon(lineIndex);
      }
    }
    _lastLineCount = currentLineCount;
  }

  void _tryAppendSemicolon(int lineIndex) {
    final lineText = _controller.codeLines[lineIndex].text;
    final trimmed = lineText.trimRight();
    if (trimmed.isEmpty) return;
    if (trimmed.endsWith(';') || trimmed.endsWith('{') || trimmed.endsWith('}')) return;
    if (trimmed.startsWith('//') || trimmed.startsWith('/*') || trimmed.startsWith('*')) return;
    if (trimmed.startsWith('#')) return;
    final needsSemicolon = RegExp(
      r'^(\s*(int|char|float|double|void|struct|enum|typedef|return|break|continue|printf|scanf|malloc|free|memcpy|memset|strlen|strcpy|strcmp|atoi|rand|srand|exit|getchar|putchar|fprintf|realloc|qsort)\b|.*[a-zA-Z_]\w*\s*=|.*\)\s*$)',
    );
    if (needsSemicolon.hasMatch(trimmed)) {
      final newLine = '$trimmed;';
      // 替换整行内容
      _controller.selectLine(lineIndex);
      _controller.replaceSelection(newLine);
    }
  }

  /// 每次操作后保持焦点和光标可见
  void _keepActive() {
    _focusNode.requestFocus();
    _controller.makeCursorCenterIfInvisible();
  }

  /// 在当前光标位置插入文本
  void insertText(String text) {
    _controller.replaceSelection(text);
    _keepActive();
  }

  /// 插入成对符号，并将光标放在中间
  void insertPair(String open, String close) {
    _controller.replaceSelection('$open$close');
    // 将光标向左移动 close.length 个字符，放在 open 和 close 之间
    for (var i = 0; i < close.length; i++) {
      _controller.moveCursor(AxisDirection.left);
    }
    _keepActive();
  }

  /// 撤销
  void undo() {
    _controller.undo();
    _keepActive();
  }

  /// 重做
  void redo() {
    _controller.redo();
    _keepActive();
  }

  /// 移动光标
  void moveCursor(int offset) {
    final direction = offset < 0 ? AxisDirection.left : AxisDirection.right;
    final count = offset.abs();
    for (var i = 0; i < count; i++) {
      _controller.moveCursor(direction);
    }
    _keepActive();
  }

  /// 滚动到指定行（1-based）
  void scrollToLine(int line) {
    if (line <= 0) return;
    final lineIndex = line - 1;
    if (lineIndex >= 0 && lineIndex < _controller.lineCount) {
      _controller.selectLine(lineIndex);
    }
  }

  /// 设置编辑器文本（用于应用修复后同步）
  void setText(String text) {
    if (_controller.text == text) return;
    _controller.removeListener(_onChanged);
    _controller.text = text;
    _controller.addListener(_onChanged);
  }

  @override
  void dispose() {
    _hideContextMenu();
    _cancelLongPress();
    _controller.removeListener(_onChanged);
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  /// 编辑器焦点节点，外部可监听焦点变化
  FocusNode get focusNode => _focusNode;

  /// 设置只读模式（用于拦截/恢复系统键盘）
  void setReadOnly(bool value) {
    if (_readOnly == value) return;
    setState(() => _readOnly = value);
  }

  // ========== 长按上下文菜单 ==========

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

  void _selectWordAt(Offset globalPosition) {
    // CodeEditor 外层是 Stack，findRenderObject 返回 RenderStack，
    // 需要通过内部私有的 _editorKey 才能拿到 _CodeFieldRender
    final codeEditorState = _codeEditorKey.currentState;
    if (codeEditorState == null) return;
    final internalKey = (codeEditorState as dynamic)._editorKey as GlobalKey?;
    final renderBox = internalKey?.currentContext?.findRenderObject() as RenderBox?;
    if (renderBox == null) return;
    // selectWord / setPositionAt 内部会自己做 globalToLocal
    final range = (renderBox as dynamic).selectWord(position: globalPosition) as CodeLineRange?;
    if (range != null) {
      _controller.selection = CodeLineSelection.fromRange(range: range);
      return;
    }
    // 空白处：定位光标并选中一个字符/空格
    final sel = (renderBox as dynamic).setPositionAt(position: globalPosition) as CodeLineSelection?;
    if (sel != null && sel.isCollapsed) {
      final index = sel.baseIndex;
      final offset = sel.baseOffset;
      final lineText = _controller.codeLines[index].text;
      if (offset < lineText.length) {
        _controller.selection = CodeLineSelection(
          baseIndex: index,
          baseOffset: offset,
          extentIndex: index,
          extentOffset: offset + 1,
        );
      } else if (offset > 0) {
        _controller.selection = CodeLineSelection(
          baseIndex: index,
          baseOffset: offset - 1,
          extentIndex: index,
          extentOffset: offset,
        );
      }
    }
  }

  void _showContextMenu(Offset position) {
    try {
      _selectWordAt(position);
    } catch (e, stack) {
      debugPrint('_selectWordAt error: $e\n$stack');
    }
    _hideContextMenu();

    final hasSelection = !_controller.selection.isCollapsed;

    // 获取选区在屏幕上的坐标
    Offset? selStart;
    Offset? selEnd;
    final codeEditorState = _codeEditorKey.currentState;
    if (codeEditorState != null) {
      try {
        final internalKey = (codeEditorState as dynamic)._editorKey as GlobalKey?;
        final renderBox = internalKey?.currentContext?.findRenderObject() as RenderBox?;
        if (renderBox != null) {
          final sel = _controller.selection;
          selStart = (renderBox as dynamic).calculateTextPositionScreenOffset(
            CodeLinePosition(index: sel.startIndex, offset: sel.startOffset),
            false,
          ) as Offset?;
          selEnd = (renderBox as dynamic).calculateTextPositionScreenOffset(
            CodeLinePosition(index: sel.endIndex, offset: sel.endOffset),
            true,
          ) as Offset?;
        }
      } catch (_) {}
    }

    // 预估宽度，判断是否需要折行
    final itemCount = (hasSelection ? 1 : 0) + 3;
    const estimatedItemWidth = 64.0;
    const estimatedDividerWidth = 1.0;
    const horizontalPadding = 32.0;
    final estimatedWidth = itemCount * estimatedItemWidth + (itemCount - 1) * estimatedDividerWidth + horizontalPadding;
    final screenWidth = MediaQuery.of(context).size.width;
    final needsWrap = estimatedWidth > screenWidth - 32;

    final overlay = Overlay.of(context);
    _contextMenuOverlay = OverlayEntry(
      builder: (context) => _ContextMenuBar(
        position: position,
        selectionStart: selStart,
        selectionEnd: selEnd,
        needsWrap: needsWrap,
        hasSelection: hasSelection,
        onCopy: hasSelection
            ? () {
                _controller.copy();
                _hideContextMenu();
              }
            : null,
        onPaste: () {
          _controller.paste();
          _hideContextMenu();
        },
        onSelectAll: () {
          _controller.selectAll();
          _hideContextMenu();
        },
        onDictionary: () {
          // TODO: 词典功能待实现
          _hideContextMenu();
        },
        onDismiss: _hideContextMenu,
      ),
    );
    overlay.insert(_contextMenuOverlay!);
  }

  /// 退格删除
  void backspace() {
    _controller.deleteBackward();
    _keepActive();
  }

  /// 插入换行（Enter）
  void insertNewline() {
    _controller.applyNewLine();
    _keepActive();
  }



  /// C 语言关键字和常用函数自动补全提示
  static final List<CodePrompt> _cAutocompletePrompts = [
    // 关键字
    const CodeKeywordPrompt(word: 'auto'),
    const CodeKeywordPrompt(word: 'break'),
    const CodeKeywordPrompt(word: 'case'),
    const CodeKeywordPrompt(word: 'char'),
    const CodeKeywordPrompt(word: 'const'),
    const CodeKeywordPrompt(word: 'continue'),
    const CodeKeywordPrompt(word: 'default'),
    const CodeKeywordPrompt(word: 'do'),
    const CodeKeywordPrompt(word: 'double'),
    const CodeKeywordPrompt(word: 'else'),
    const CodeKeywordPrompt(word: 'enum'),
    const CodeKeywordPrompt(word: 'extern'),
    const CodeKeywordPrompt(word: 'float'),
    const CodeKeywordPrompt(word: 'for'),
    const CodeKeywordPrompt(word: 'goto'),
    const CodeKeywordPrompt(word: 'if'),
    const CodeKeywordPrompt(word: 'int'),
    const CodeKeywordPrompt(word: 'long'),
    const CodeKeywordPrompt(word: 'register'),
    const CodeKeywordPrompt(word: 'return'),
    const CodeKeywordPrompt(word: 'short'),
    const CodeKeywordPrompt(word: 'signed'),
    const CodeKeywordPrompt(word: 'sizeof'),
    const CodeKeywordPrompt(word: 'static'),
    const CodeKeywordPrompt(word: 'struct'),
    const CodeKeywordPrompt(word: 'switch'),
    const CodeKeywordPrompt(word: 'typedef'),
    const CodeKeywordPrompt(word: 'union'),
    const CodeKeywordPrompt(word: 'unsigned'),
    const CodeKeywordPrompt(word: 'void'),
    const CodeKeywordPrompt(word: 'volatile'),
    const CodeKeywordPrompt(word: 'while'),
    // 标准库函数
    const CodeFunctionPrompt(word: 'printf', type: 'int', parameters: {'format': 'const char*'}),
    const CodeFunctionPrompt(word: 'scanf', type: 'int', parameters: {'format': 'const char*'}),
    const CodeFunctionPrompt(word: 'malloc', type: 'void*', parameters: {'size': 'size_t'}),
    const CodeFunctionPrompt(word: 'free', type: 'void', parameters: {'ptr': 'void*'}),
    const CodeFunctionPrompt(word: 'memset', type: 'void*', parameters: {'s': 'void*', 'c': 'int', 'n': 'size_t'}),
    const CodeFunctionPrompt(word: 'memcpy', type: 'void*', parameters: {'dest': 'void*', 'src': 'const void*', 'n': 'size_t'}),
    const CodeFunctionPrompt(word: 'strlen', type: 'size_t', parameters: {'s': 'const char*'}),
    const CodeFunctionPrompt(word: 'strcpy', type: 'char*', parameters: {'dest': 'char*', 'src': 'const char*'}),
    const CodeFunctionPrompt(word: 'strcmp', type: 'int', parameters: {'s1': 'const char*', 's2': 'const char*'}),
    const CodeFunctionPrompt(word: 'atoi', type: 'int', parameters: {'str': 'const char*'}),
    const CodeFunctionPrompt(word: 'rand', type: 'int', parameters: {}),
    const CodeFunctionPrompt(word: 'srand', type: 'void', parameters: {'seed': 'unsigned int'}),
    const CodeFunctionPrompt(word: 'exit', type: 'void', parameters: {'status': 'int'}),
    const CodeFunctionPrompt(word: 'getchar', type: 'int', parameters: {}),
    const CodeFunctionPrompt(word: 'putchar', type: 'int', parameters: {'c': 'int'}),
    const CodeFunctionPrompt(word: 'fprintf', type: 'int', parameters: {'stream': 'FILE*', 'format': 'const char*'}),
    const CodeFunctionPrompt(word: 'qsort', type: 'void', parameters: {'base': 'void*', 'nmemb': 'size_t', 'size': 'size_t', 'compar': 'int (*)()'}),
    const CodeFunctionPrompt(word: 'realloc', type: 'void*', parameters: {'ptr': 'void*', 'size': 'size_t'}),
    // 常用宏
    const CodeKeywordPrompt(word: 'NULL'),
    const CodeKeywordPrompt(word: 'EOF'),
    const CodeKeywordPrompt(word: 'stdout'),
    const CodeKeywordPrompt(word: 'stderr'),
    const CodeKeywordPrompt(word: 'true'),
    const CodeKeywordPrompt(word: 'false'),
    // 控制结构模板
    const CodeKeywordPrompt(word: 'main'),
    const CodeKeywordPrompt(word: 'include'),
    const CodeKeywordPrompt(word: 'define'),
  ];

  int _currentHighlightLine = 0;
  List<rust_unified.AccessedVar> _currentAccessedVars = [];

  /// re_editor spanBuilder：在当前执行行的变量名上添加底色高亮。
  TextSpan _buildVariableHighlightSpan({
    required BuildContext context,
    required int index,
    required CodeLine codeLine,
    required TextSpan textSpan,
    required TextStyle style,
  }) {
    final line = index + 1;
    if (line != _currentHighlightLine || _currentAccessedVars.isEmpty) {
      return textSpan;
    }

    final highlights = _currentAccessedVars.map((a) => _VarHighlight(
      name: a.name,
      color: a.accessType == 'Write'
          ? Colors.orange.withValues(alpha: 0.25)
          : Colors.blueAccent.withValues(alpha: 0.15),
    )).toList();

    final newChildren = _applyHighlightsToSpan(textSpan, highlights);
    return TextSpan(
      style: textSpan.style,
      children: newChildren,
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(ideProvider);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;
    final unifiedState = ref.watch(unifiedProvider);

    // 更新变量级高亮状态
    int newHighlightLine = 0;
    List<rust_unified.AccessedVar> newAccessedVars = [];
    if (unifiedState.currentStep >= 0 &&
        unifiedState.currentStep < unifiedState.frameCache.length) {
      final payload = unifiedState.frameCache[unifiedState.currentStep];
      newHighlightLine = payload.codeLine;
      newAccessedVars = payload.accessedVars;
    }
    final hasHighlightChanged = newHighlightLine != _currentHighlightLine ||
        !_accessedVarsEqual(newAccessedVars, _currentAccessedVars);
    if (hasHighlightChanged) {
      _currentHighlightLine = newHighlightLine;
      _currentAccessedVars = List.from(newAccessedVars);
      _controller.forceRepaint();
    }

    // 调试时自动聚焦当前行
    if (state.isStepMode && state.currentLine > 0) {
      SchedulerBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          final lineIndex = state.currentLine - 1;
          if (lineIndex >= 0 && lineIndex < _controller.lineCount) {
            _controller.selectLine(lineIndex);
          }
        }
      });
    }

    final editorBackground = isDark ? const Color(0xff282c34) : const Color(0xfffafafa);
    final editorTextColor = isDark ? const Color(0xffabb2bf) : const Color(0xff383a42);
    final separatorColor = isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5);
    final cursorLineColor = state.isStepMode || state.isRunning
        ? Colors.blueAccent.withValues(alpha: 0.3)
        : null;

    // 拦截 warm-up frame 中可能出现的 0/Infinity 高度约束，
    // 避免 re_editor 的 DefaultCodeLineNumber 触发 assert 崩溃
    return LayoutBuilder(
      builder: (context, constraints) {
        if (constraints.maxHeight <= 0 || constraints.maxHeight == double.infinity) {
          return Container(
            decoration: BoxDecoration(
              color: editorBackground,
              borderRadius: BorderRadius.circular(8),
            ),
          );
        }
        return Container(
          decoration: BoxDecoration(
            color: editorBackground,
            borderRadius: BorderRadius.circular(8),
          ),
          child: CodeAutocomplete(
        viewBuilder: (context, notifier, onSelected) {
          return _AutocompleteListView(
            notifier: notifier,
            onSelected: onSelected,
            isDark: isDark,
          );
        },
        promptsBuilder: DefaultCodeAutocompletePromptsBuilder(
          keywordPrompts: _cAutocompletePrompts.whereType<CodeKeywordPrompt>().toList(),
          directPrompts: _cAutocompletePrompts.whereType<CodeFunctionPrompt>().toList(),
        ),
        child: Listener(
          onPointerDown: (event) {
            _swipeStart = event.position;
            _startLongPress(event.position);
          },
          onPointerMove: (event) => _checkLongPressMove(event.position),
          onPointerUp: (event) {
            final wasShortPress = _longPressTimer != null;
            // 立即取消长按计时器，避免后续操作耗时导致误触发长按
            _cancelLongPress();

            // 检测上下滑动手势收起键盘（使用独立的 _swipeStart，不受长按取消影响）
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

            // 短按：延迟到 re_editor 内部更新 selection 后再判断空白处
            if (wasShortPress) {
              WidgetsBinding.instance.addPostFrameCallback((_) {
                if (!mounted) return;
                final sel = _controller.selection;
                final index = sel.baseIndex;
                if (index >= 0 && index < _controller.lineCount) {
                  final lineText = _controller.codeLines[index].text;
                  final offset = sel.baseOffset;
                  // 空行、行尾之后、尾部空白区域 → 空白处
                  final isBlank = lineText.trim().isEmpty ||
                      offset >= lineText.length ||
                      offset >= lineText.trimRight().length;
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
          child: CodeEditor(
            key: _codeEditorKey,
            controller: _controller,
            focusNode: _focusNode,
            readOnly: _readOnly,
            showCursorWhenReadOnly: true,
            style: CodeEditorStyle(
              fontSize: 14,
              fontFamily: 'Consolas',
              fontFamilyFallback: const ['monospace'],
              textColor: editorTextColor,
              backgroundColor: editorBackground,
              cursorColor: isDark ? Colors.white : Colors.black,
              cursorWidth: 2,
              cursorLineColor: cursorLineColor,
              codeTheme: CodeHighlightTheme(
                languages: {'c': CodeHighlightThemeMode(mode: langC)},
                theme: isDark ? atomOneDarkTheme : atomOneLightTheme,
              ),
            ),
            indicatorBuilder: (context, editingController, chunkController, notifier) =>
                _buildGutter(editingController, notifier, state, isDark),
            scrollbarBuilder: (context, child, details) =>
                Scrollbar(controller: details.controller, child: child),
            sperator: Container(width: 1, color: separatorColor),
          ),
        ),
      ),
    );
  },
);
  }

  Widget _buildGutter(
    CodeLineEditingController editingController,
    CodeIndicatorValueNotifier notifier,
    IdeState state,
    bool isDark,
  ) {
    // 按行号分组诊断（取最严重的）
    final diagMap = <int, int>{}; // line -> severity (0=error, 1=warning, 2=hint)
    for (final d in state.diagnostics) {
      if (!diagMap.containsKey(d.line) || d.severity < diagMap[d.line]!) {
        diagMap[d.line] = d.severity;
      }
    }

    final lineNumberColor = isDark ? const Color(0xff5c6370) : const Color(0xffa0a1a7);
    final focusedLineNumberColor = isDark ? const Color(0xffabb2bf) : const Color(0xff383a42);

    final unifiedState = ref.watch(unifiedProvider);
    final heatmap = unifiedState.heatmap;

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 执行路径热力图条带
        if (heatmap != null && heatmap.lineCounts.isNotEmpty)
          _HeatmapGutterStrip(
            notifier: notifier,
            heatmap: heatmap,
            isDark: isDark,
          ),
        DefaultCodeLineNumber(
          controller: editingController,
          notifier: notifier,
          textStyle: TextStyle(color: lineNumberColor, fontSize: 14),
          focusedTextStyle: TextStyle(color: focusedLineNumberColor, fontSize: 14),
          customLineIndex2Text: (lineIndex) {
            final line = lineIndex + 1;
            final hasBreakpoint = state.breakpoints.contains(line);
            final severity = diagMap[line];

            String prefix = '';
            if (hasBreakpoint) {
              prefix = '● ';
            } else if (severity == 0) {
              prefix = '✗ ';
            } else if (severity == 1) {
              prefix = '⚠ ';
            } else if (severity == 2) {
              prefix = 'ℹ ';
            }

            // 当前调试行特殊标记
            if (state.isStepMode && line == state.currentLine) {
              return '$prefix▶ $line';
            }

            // 统一模式变量访问指示（行号旁显示 R/W 标记）
            String varSuffix = '';
            if (unifiedState.canSeek &&
                unifiedState.currentStep >= 0 &&
                unifiedState.currentStep < unifiedState.frameCache.length) {
              final payload = unifiedState.frameCache[unifiedState.currentStep];
              if (payload.codeLine == line && payload.accessedVars.isNotEmpty) {
                final markers = payload.accessedVars.take(2).map((a) {
                  final marker = a.accessType == 'Read' ? 'R' : 'W';
                  return '${a.name}=$marker';
                }).join(' ');
                varSuffix = ' $markers';
              }
            }

            return prefix.isEmpty
                ? '$line$varSuffix'
                : '$prefix$line$varSuffix';
          },
        ),
      ],
    );
  }
}

/// 编辑器左侧热力图条带。
///
/// 监听 [CodeIndicatorValueNotifier] 获取每行的渲染位置，
/// 在左侧 4px 宽度内绘制颜色深浅表示执行次数。
class _HeatmapGutterStrip extends StatefulWidget {
  final CodeIndicatorValueNotifier notifier;
  final rust_unified.HeatmapData heatmap;
  final bool isDark;

  const _HeatmapGutterStrip({
    required this.notifier,
    required this.heatmap,
    required this.isDark,
  });

  @override
  State<_HeatmapGutterStrip> createState() => _HeatmapGutterStripState();
}

class _HeatmapGutterStripState extends State<_HeatmapGutterStrip> {
  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder<CodeIndicatorValue?>(
      valueListenable: widget.notifier,
      builder: (context, value, child) {
        if (value == null) {
          return const SizedBox(width: 4);
        }

        final lineCountMap = <int, int>{};
        for (final entry in widget.heatmap.lineCounts) {
          lineCountMap[entry.$1] = entry.$2.toInt();
        }
        final maxCount = widget.heatmap.maxCount.toInt() > 0 ? widget.heatmap.maxCount.toInt() : 1;

        return CustomPaint(
          painter: _HeatmapPainter(
            paragraphs: value.paragraphs,
            lineCountMap: lineCountMap,
            maxCount: maxCount,
            isDark: widget.isDark,
          ),
          size: const Size(4, double.infinity),
        );
      },
    );
  }
}

class _HeatmapPainter extends CustomPainter {
  final List<CodeLineRenderParagraph> paragraphs;
  final Map<int, int> lineCountMap;
  final int maxCount;
  final bool isDark;

  _HeatmapPainter({
    required this.paragraphs,
    required this.lineCountMap,
    required this.maxCount,
    required this.isDark,
  });

  @override
  void paint(Canvas canvas, Size size) {
    for (int i = 0; i < paragraphs.length; i++) {
      final line = i + 1;
      final count = lineCountMap[line] ?? 0;
      if (count == 0) continue;

      final intensity = count / maxCount;
      final color = _heatmapColor(intensity);

      final rect = Rect.fromLTWH(
        0,
        paragraphs[i].offset.dy,
        size.width,
        paragraphs[i].height,
      );
      final rrect = RRect.fromRectAndRadius(rect, const Radius.circular(1));
      canvas.drawRRect(rrect, Paint()..color = color);
    }
  }

  Color _heatmapColor(double intensity) {
    // 从浅灰到深红的渐变
    if (intensity < 0.2) {
      return isDark ? const Color(0xFF3A3A3C) : const Color(0xFFE0E0E0);
    } else if (intensity < 0.4) {
      return isDark ? const Color(0xFF5C3A3A) : const Color(0xFFFFCDD2);
    } else if (intensity < 0.6) {
      return isDark ? const Color(0xFF7A3A3A) : const Color(0xFFEF9A9A);
    } else if (intensity < 0.8) {
      return isDark ? const Color(0xFFB04A4A) : const Color(0xFFE57373);
    } else {
      return isDark ? const Color(0xFFD32F2F) : const Color(0xFFC62828);
    }
  }

  @override
  bool shouldRepaint(covariant _HeatmapPainter old) {
    return old.lineCountMap != lineCountMap ||
        old.maxCount != maxCount ||
        old.paragraphs.length != paragraphs.length;
  }
}

/// 自动补全下拉列表
class _AutocompleteListView extends StatelessWidget implements PreferredSizeWidget {
  final ValueNotifier<CodeAutocompleteEditingValue> notifier;
  final ValueChanged<CodeAutocompleteResult> onSelected;
  final bool isDark;

  const _AutocompleteListView({
    required this.notifier,
    required this.onSelected,
    required this.isDark,
  });

  @override
  Size get preferredSize => const Size(200, 200);

  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder<CodeAutocompleteEditingValue>(
      valueListenable: notifier,
      builder: (context, value, child) {
        return Container(
          constraints: const BoxConstraints(maxHeight: 200),
          decoration: BoxDecoration(
            color: isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff),
            border: Border.all(color: isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5)),
            borderRadius: BorderRadius.circular(4),
          ),
          child: Material(
            color: Colors.transparent,
            child: ListView.builder(
              shrinkWrap: true,
              itemCount: value.prompts.length,
              itemBuilder: (context, index) {
                final prompt = value.prompts[index];
                final isSelected = index == value.index;
                return InkWell(
                  onTap: () => onSelected(value.autocomplete),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                    decoration: BoxDecoration(
                      color: isSelected ? Colors.blueAccent.withValues(alpha: 0.3) : null,
                    ),
                    child: Row(
                      children: [
                        _PromptIcon(prompt: prompt),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            prompt.word,
                            style: TextStyle(
                              fontSize: 13,
                              color: isSelected
                                  ? Colors.white
                                  : isDark
                                      ? const Color(0xffd4d4d4)
                                      : const Color(0xff383a42),
                              fontFamily: 'monospace',
                            ),
                          ),
                        ),
                        if (prompt is CodeFunctionPrompt)
                          Text(
                            prompt.type,
                            style: const TextStyle(fontSize: 11, color: Colors.grey, fontFamily: 'monospace'),
                          ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
        );
      },
    );
  }
}

class _PromptIcon extends StatelessWidget {
  final CodePrompt prompt;

  const _PromptIcon({required this.prompt});

  @override
  Widget build(BuildContext context) {
    if (prompt is CodeFunctionPrompt) {
      return const Icon(Icons.functions, size: 14, color: Colors.yellowAccent);
    }
    if (prompt is CodeFieldPrompt) {
      return const Icon(Icons.data_object, size: 14, color: Colors.cyanAccent);
    }
    return const Icon(Icons.text_fields, size: 14, color: Colors.blueAccent);
  }
}

/// 长按横向上下文菜单
class _ContextMenuBar extends StatelessWidget {
  final Offset position;
  final Offset? selectionStart;
  final Offset? selectionEnd;
  final bool needsWrap;
  final bool hasSelection;
  final VoidCallback? onCopy;
  final VoidCallback? onPaste;
  final VoidCallback? onSelectAll;
  final VoidCallback? onDictionary;
  final VoidCallback onDismiss;

  const _ContextMenuBar({
    required this.position,
    this.selectionStart,
    this.selectionEnd,
    required this.needsWrap,
    required this.hasSelection,
    this.onCopy,
    this.onPaste,
    this.onSelectAll,
    this.onDictionary,
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

    // 以选区中心为锚点；若无选区坐标则回退到触摸点
    final selStart = selectionStart;
    final selEnd = selectionEnd;
    final centerX = selStart != null && selEnd != null
        ? (selStart.dx + selEnd.dx) / 2
        : position.dx;
    final selectionTop = selStart?.dy ?? position.dy;
    final selectionBottom = selEnd?.dy ?? position.dy;

    // 优先放选区上方
    double top = selectionTop - menuHeight - 8;
    if (top < safeTop + 8) {
      top = selectionBottom + 8;
    }
    // 防止超出底部（如键盘区域）
    if (top + menuHeight > screenSize.height - safeBottom - 8) {
      top = selectionTop - menuHeight - 8;
    }

    // 预估宽度用于居中计算
    final itemCount = (hasSelection ? 1 : 0) + 3;
    const estimatedItemWidth = 64.0;
    const estimatedDividerWidth = 1.0;
    const horizontalPadding = 32.0;
    final estimatedWidth = needsWrap
        ? ((itemCount + 1) ~/ 2) * estimatedItemWidth + (((itemCount + 1) ~/ 2) - 1) * estimatedDividerWidth + horizontalPadding
        : itemCount * estimatedItemWidth + (itemCount - 1) * estimatedDividerWidth + horizontalPadding;

    double left = centerX - estimatedWidth / 2;
    if (left < 16) left = 16;
    if (left + estimatedWidth > screenSize.width - 16) {
      left = screenSize.width - estimatedWidth - 16;
    }
    if (left < 16) left = 16;

    Widget buildMenuRow(List<Widget> items) {
      return IntrinsicWidth(
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: items,
        ),
      );
    }

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
      if (onDictionary != null) {
        items.add(_MenuButton(text: '词典', onTap: onDictionary!, textColor: textColor));
      }
      if (items.isNotEmpty && items.last is _MenuDivider) {
        items.removeLast();
      }
      return items;
    }

    final allItems = buildItems();
    Widget menuContent;
    if (needsWrap && allItems.length > 2) {
      final half = (allItems.length / 2).ceil();
      menuContent = Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          buildMenuRow(allItems.sublist(0, half)),
          Container(height: 1, color: dividerColor, margin: const EdgeInsets.symmetric(horizontal: 8)),
          buildMenuRow(allItems.sublist(half)),
        ],
      );
    } else {
      menuContent = buildMenuRow(allItems);
    }

    return Stack(
      children: [
        Positioned.fill(
          child: GestureDetector(
            onTap: onDismiss,
            behavior: HitTestBehavior.translucent,
            child: Container(color: Colors.transparent),
          ),
        ),
        Positioned(
          left: left,
          top: top,
          child: Material(
            color: Colors.transparent,
            child: Container(
              constraints: BoxConstraints(
                maxWidth: screenSize.width - 32,
                minHeight: needsWrap ? 40 : 44,
              ),
              decoration: BoxDecoration(
                color: bgColor,
                borderRadius: BorderRadius.circular(10),
              ),
              child: menuContent,
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
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(10),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Text(
          text,
          style: TextStyle(color: textColor, fontSize: 14),
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
      height: 20,
      color: color,
      margin: const EdgeInsets.symmetric(vertical: 10),
    );
  }
}

// ========== 变量级高亮辅助类（Phase 7） ==========

class _VarHighlight {
  final String name;
  final Color color;
  const _VarHighlight({required this.name, required this.color});
}

class _HighlightMatch {
  final int start;
  final int end;
  final Color color;
  const _HighlightMatch({required this.start, required this.end, required this.color});
}

List<InlineSpan> _applyHighlightsToSpan(InlineSpan span, List<_VarHighlight> highlights) {
  if (span is! TextSpan) return [span];

  final text = span.text;
  if (text != null && text.isNotEmpty) {
    return _splitTextSpan(span, text, highlights);
  }

  if (span.children == null || span.children!.isEmpty) {
    return [span];
  }

  final newChildren = span.children!
      .expand((child) => _applyHighlightsToSpan(child, highlights))
      .toList();

  return [
    TextSpan(
      style: span.style,
      recognizer: span.recognizer,
      mouseCursor: span.mouseCursor,
      onEnter: span.onEnter,
      onExit: span.onExit,
      semanticsLabel: span.semanticsLabel,
      locale: span.locale,
      spellOut: span.spellOut,
      children: newChildren,
    ),
  ];
}

List<InlineSpan> _splitTextSpan(TextSpan span, String text, List<_VarHighlight> highlights) {
  final matches = <_HighlightMatch>[];
  for (final h in highlights) {
    final pattern = RegExp(r'\b' + RegExp.escape(h.name) + r'\b');
    for (final m in pattern.allMatches(text)) {
      matches.add(_HighlightMatch(start: m.start, end: m.end, color: h.color));
    }
  }

  if (matches.isEmpty) return [span];

  matches.sort((a, b) => a.start.compareTo(b.start));
  final merged = <_HighlightMatch>[];
  for (final m in matches) {
    if (merged.isEmpty || m.start >= merged.last.end) {
      merged.add(m);
    } else if (m.end > merged.last.end) {
      merged.last = _HighlightMatch(
        start: merged.last.start,
        end: m.end,
        color: merged.last.color,
      );
    }
  }

  final result = <InlineSpan>[];
  int pos = 0;
  for (final m in merged) {
    if (m.start > pos) {
      result.add(TextSpan(text: text.substring(pos, m.start), style: span.style));
    }
    result.add(TextSpan(
      text: text.substring(m.start, m.end),
      style: (span.style ?? const TextStyle()).copyWith(
        backgroundColor: m.color,
        fontWeight: FontWeight.w600,
      ),
    ));
    pos = m.end;
  }
  if (pos < text.length) {
    result.add(TextSpan(text: text.substring(pos), style: span.style));
  }
  return result;
}

bool _accessedVarsEqual(List<rust_unified.AccessedVar> a, List<rust_unified.AccessedVar> b) {
  if (a.length != b.length) return false;
  for (int i = 0; i < a.length; i++) {
    if (a[i].name != b[i].name || a[i].accessType != b[i].accessType) return false;
  }
  return true;
}
