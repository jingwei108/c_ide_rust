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

class EditorPanel extends ConsumerStatefulWidget {
  final VoidCallback? onTap;

  const EditorPanel({super.key, this.onTap});

  @override
  ConsumerState<EditorPanel> createState() => EditorPanelState();
}

class EditorPanelState extends ConsumerState<EditorPanel> {
  late CodeLineEditingController _controller;
  final _focusNode = FocusNode();
  bool _readOnly = true;

  @override
  void initState() {
    super.initState();
    final source = ref.read(ideProvider).source;
    _controller = CodeLineEditingController.fromText(
      source,
      const CodeLineOptions(indentSize: 4),
    );
    _lastLineCount = _controller.lineCount;
    _controller.addListener(_onChanged);
  }

  int _lastLineCount = 0;

  void _onChanged() {
    ref.read(ideProvider.notifier).updateSource(_controller.text);

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

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(ideProvider);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;

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
          onPointerDown: (_) => widget.onTap?.call(),
          behavior: HitTestBehavior.translucent,
          child: CodeEditor(
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

    return DefaultCodeLineNumber(
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

        return prefix.isEmpty ? '$line' : '$prefix$line';
      },
    );
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
