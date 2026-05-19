import 'dart:math';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'cide_document.dart';
import 'editor_painter.dart';
import 'editor_layers.dart';

/// ---------------------------------------------------------------------------
/// CideEditor — Gesture Proxy 模式代码编辑器
/// ---------------------------------------------------------------------------
/// 架构：
/// - EditableText：完全透明，只作为手势代理 + IME 代理
/// - CustomPaint：负责所有可见渲染（文本、选区、语法高亮、运行时高亮等）
/// - SingleChildScrollView：统一滚动控制
/// ---------------------------------------------------------------------------

class CideEditor extends StatefulWidget {
  final CideDocument document;
  final List<EditorLayer> layers;
  final bool readOnly;
  final VoidCallback? onTap;
  final void Function(Offset position)? onPointerDown;
  final TextStyle style;
  final void Function(int lineIndex, String lineText)? onTryAppendSemicolon;

  const CideEditor({
    super.key,
    required this.document,
    this.layers = const [],
    this.readOnly = false,
    this.onTap,
    this.onPointerDown,
    required this.style,
    this.onTryAppendSemicolon,
  });

  @override
  State<CideEditor> createState() => CideEditorState();
}

class CideEditorState extends State<CideEditor>
    implements TextInputClient {
  // ---------------------------------------------------------------------------
  // 控制器
  // ---------------------------------------------------------------------------
  late final TextEditingController _proxyController;
  late final FocusNode _focusNode;
  late final ScrollController _scrollController;

  // TextInputConnection 生命周期
  TextInputConnection? _inputConnection;
  bool _isSystemKeyboardActive = false;

  // 同步锁（防止双向循环）
  bool _syncing = false;

  // 视口尺寸（用于 CustomPaint 裁剪）
  Size _viewportSize = Size.zero;

  // 只读状态（可被外部修改）
  bool _readOnly = false;

  // ---------------------------------------------------------------------------
  // 生命周期
  // ---------------------------------------------------------------------------
  @override
  void initState() {
    super.initState();
    _readOnly = widget.readOnly;
    _proxyController = TextEditingController(text: widget.document.text);
    _focusNode = FocusNode();
    _scrollController = ScrollController();

    _proxyController.addListener(_onProxyChanged);
    widget.document.addListener(_onDocumentChanged);
    _focusNode.addListener(_onFocusChanged);
    _scrollController.addListener(_onScrollChanged);
  }

  @override
  void didUpdateWidget(covariant CideEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.document != widget.document) {
      oldWidget.document.removeListener(_onDocumentChanged);
      widget.document.addListener(_onDocumentChanged);
      _syncToProxy();
    }
  }

  @override
  void dispose() {
    _proxyController.removeListener(_onProxyChanged);
    widget.document.removeListener(_onDocumentChanged);
    _focusNode.removeListener(_onFocusChanged);
    _scrollController.removeListener(_onScrollChanged);

    _detachInputConnection();
    _proxyController.dispose();
    _focusNode.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  // ---------------------------------------------------------------------------
  // 焦点 & TextInputConnection
  // ---------------------------------------------------------------------------
  void _onFocusChanged() {
    if (_focusNode.hasFocus) {
      if (_isSystemKeyboardActive) {
        _attachInputConnection();
      }
    } else {
      _detachInputConnection();
    }
  }

  void _attachInputConnection() {
    if (_inputConnection != null || _readOnly) return;
    _inputConnection = TextInput.attach(
      this,
      const TextInputConfiguration(
        inputType: TextInputType.multiline,
        inputAction: TextInputAction.newline,
        autocorrect: false,
        enableSuggestions: false,
        enableIMEPersonalizedLearning: false,
      ),
    );
    _inputConnection!.show();
    _inputConnection!.setEditingState(_proxyController.value);
  }

  void _detachInputConnection() {
    _inputConnection?.close();
    _inputConnection = null;
  }

  /// 焦点节点
  FocusNode get focusNode => _focusNode;

  /// 滚动控制器（外部可用于同步 Gutter 等）
  ScrollController get scrollController => _scrollController;

  /// 切换到系统键盘模式（桌面/平板物理键盘）
  void showSystemKeyboard() {
    if (_isSystemKeyboardActive) return;
    _isSystemKeyboardActive = true;
    widget.document.clearUndoStack();
    if (_focusNode.hasFocus) {
      _attachInputConnection();
    }
  }

  void setReadOnly(bool value) {
    if (_readOnly == value) return;
    setState(() {
      _readOnly = value;
      if (_readOnly) {
        _detachInputConnection();
      } else if (_focusNode.hasFocus && _isSystemKeyboardActive) {
        _attachInputConnection();
      }
    });
  }

  /// 切换到自绘键盘模式（移动端）
  void showCustomKeyboard() {
    if (!_isSystemKeyboardActive) return;
    _isSystemKeyboardActive = false;
    _detachInputConnection();
  }

  // ---------------------------------------------------------------------------
  // 滚动
  // ---------------------------------------------------------------------------
  void _onScrollChanged() {
    setState(() {}); // 触发 CustomPaint 重绘（viewport 变化）
  }

  double get _totalContentHeight {
    final lineHeight = widget.style.fontSize != null
        ? widget.style.fontSize! * 1.5
        : 21.0;
    return widget.document.lineCount * lineHeight;
  }

  // ---------------------------------------------------------------------------
  // 双向同步：Proxy → Document
  // ---------------------------------------------------------------------------
  void _onProxyChanged() {
    if (_syncing) return;
    _syncing = true;

    final proxy = _proxyController.value;
    final oldText = widget.document.text;

    final newSelection = DocSelection(
      base: widget.document.offsetToPosition(proxy.selection.baseOffset),
      extent: widget.document.offsetToPosition(proxy.selection.extentOffset),
    );

    // 文本差异同步（与选区/composing 批量更新，避免中间状态干扰 IME）
    if (proxy.text != oldText) {
      final diff = CideDocument.computeDiff(oldText, proxy.text);
      if (diff != null) {
        widget.document.applyEditSync(diff, newSelection, proxy.composing);
      } else {
        widget.document.setTextSync(proxy.text, newSelection, proxy.composing);
      }
    } else {
      // 仅选区 / composing 变化
      widget.document.updateSelection(newSelection);
      widget.document.updateComposing(proxy.composing);
    }

    _syncing = false;
  }

  // ---------------------------------------------------------------------------
  // 双向同步：Document → Proxy
  // ---------------------------------------------------------------------------
  void _onDocumentChanged() {
    if (_syncing) return;
    _syncing = true;

    _proxyController.value = TextEditingValue(
      text: widget.document.text,
      selection: _toTextSelection(widget.document.selection),
      composing: widget.document.composing,
    );

    // 注意：不要在这里调用 _inputConnection?.setEditingState()。
    // IME 发起的编辑（updateEditingValue）不需要回传，否则会造成
    // 反馈循环，导致候选词中断、光标跳回左上角。
    // 自定义键盘操作后若需同步 IME，在对应 API 中单独处理。

    _syncing = false;

    if (mounted) setState(() {});
  }

  void _syncToProxy() {
    _proxyController.value = TextEditingValue(
      text: widget.document.text,
      selection: _toTextSelection(widget.document.selection),
      composing: widget.document.composing,
    );
  }

  TextSelection _toTextSelection(DocSelection sel) {
    return TextSelection(
      baseOffset: widget.document.positionToOffset(sel.base),
      extentOffset: widget.document.positionToOffset(sel.extent),
    );
  }

  // ---------------------------------------------------------------------------
  // 自绘键盘 API
  // ---------------------------------------------------------------------------
  void insertText(String text) {
    final sel = widget.document.selection;
    final offset = widget.document.positionToOffset(sel.base);
    final op = widget.document.createInsertOp(offset, text);
    widget.document.applyEdit(op);
    // 移动光标到插入文本之后
    final newOffset = offset + text.length;
    final newPos = widget.document.offsetToPosition(newOffset);
    widget.document.updateSelection(
      DocSelection(base: newPos, extent: newPos),
    );
    _syncToProxy();
  }

  void backspace() {
    final sel = widget.document.selection;
    if (!sel.isCollapsed) {
      // 删除选区
      final start = widget.document.positionToOffset(sel.start);
      final end = widget.document.positionToOffset(sel.end);
      final op = EditOp(
        startOffset: start,
        oldText: widget.document.text.substring(start, end),
        newText: '',
      );
      widget.document.applyEdit(op);
      final newPos = widget.document.offsetToPosition(start);
      widget.document.updateSelection(
        DocSelection(base: newPos, extent: newPos),
      );
    } else {
      // 删除前一个字符
      final offset = widget.document.positionToOffset(sel.base);
      if (offset <= 0) return;
      final op = widget.document.createDeleteOp(offset - 1, 1);
      widget.document.applyEdit(op);
      final newPos = widget.document.offsetToPosition(offset - 1);
      widget.document.updateSelection(
        DocSelection(base: newPos, extent: newPos),
      );
    }
    _syncToProxy();
  }

  void moveCursor(int delta) {
    final sel = widget.document.selection;
    final offset = widget.document.positionToOffset(sel.base);
    final newOffset = (offset + delta).clamp(0, widget.document.text.length);
    final newPos = widget.document.offsetToPosition(newOffset);
    widget.document.updateSelection(
      DocSelection(base: newPos, extent: newPos),
    );
    _syncToProxy();
  }

  void undo() => widget.document.undo();
  void redo() => widget.document.redo();

  /// 获取当前光标所在行号（1-based），无焦点时返回 0
  int getCurrentLine() {
    final line = widget.document.selection.base.line;
    if (line < 0) return 0;
    return line + 1;
  }

  /// 插入换行（Enter）
  void insertNewline() {
    insertText('\n');
    // VS 风格：尝试为前一行补分号
    final line = widget.document.selection.base.line;
    if (line > 0) {
      final prevLineText = widget.document.lineText(line - 1);
      widget.onTryAppendSemicolon?.call(line - 1, prevLineText);
    }
  }

  /// 插入成对符号，并将光标放在中间
  void insertPair(String open, String close) {
    insertText('$open$close');
    // 将光标向左移动 close.length 个字符
    moveCursor(-close.length);
  }

  /// 滚动到指定行（1-based）
  void scrollToLine(int line) {
    if (line <= 0) return;
    final lineIndex = line - 1;
    if (lineIndex < 0 || lineIndex >= widget.document.lineCount) return;
    final lineHeight = widget.style.fontSize != null
        ? widget.style.fontSize! * 1.5
        : 21.0;
    final targetOffset = lineIndex * lineHeight;
    _scrollController.animateTo(
      targetOffset,
      duration: const Duration(milliseconds: 200),
      curve: Curves.easeOut,
    );
  }

  /// 全选
  void selectAll() {
    final endLine = widget.document.lineCount - 1;
    final endCol = widget.document.lineText(endLine).length;
    widget.document.updateSelection(
      DocSelection(
        base: const DocPosition(line: 0, col: 0),
        extent: DocPosition(line: endLine, col: endCol),
      ),
    );
    _syncToProxy();
  }

  /// 复制（选区 → 剪贴板）
  Future<void> copy() async {
    final sel = widget.document.selection;
    if (sel.isCollapsed) return;
    final start = widget.document.positionToOffset(sel.start);
    final end = widget.document.positionToOffset(sel.end);
    final text = widget.document.text.substring(start, end);
    await Clipboard.setData(ClipboardData(text: text));
  }

  /// 粘贴（剪贴板 → 光标处）
  Future<void> paste() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    if (data?.text != null) {
      insertText(data!.text!);
    }
  }

  // ---------------------------------------------------------------------------
  // TextInputClient 实现
  // ---------------------------------------------------------------------------
  @override
  AutofillScope? get currentAutofillScope => null;

  @override
  void updateEditingValue(TextEditingValue value) {
    if (_syncing) return;
    _proxyController.value = value;
  }

  @override
  void updateFloatingCursor(RawFloatingCursorPoint point) {}

  @override
  void connectionClosed() {
    _inputConnection = null;
  }

  @override
  void performAction(TextInputAction action) {}

  @override
  void performPrivateCommand(String action, Map<String, dynamic> data) {}

  @override
  void showAutocorrectionPromptRect(int start, int end) {}

  @override
  void insertTextPlaceholder(Size size) {}

  @override
  void removeTextPlaceholder() {}

  @override
  void didChangeInputControl(
      TextInputControl? oldControl, TextInputControl? newControl) {}

  @override
  void insertContent(KeyboardInsertedContent content) {
    if (content.hasData) {
      // 图片等内容暂不支持，仅处理文本由 onChanged 接管
    }
  }

  @override
  void performSelector(String selectorName) {}

  @override
  void showToolbar() {}

  @override
  TextEditingValue? get currentTextEditingValue => _proxyController.value;

  // ---------------------------------------------------------------------------
  // 构建
  // ---------------------------------------------------------------------------
  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        _viewportSize = constraints.biggest;
        return SingleChildScrollView(
          controller: _scrollController,
          child: SizedBox(
            height: max(_totalContentHeight, _viewportSize.height),
            child: Stack(
              children: [
                // 层 1：EditableText（透明代理）
                Positioned.fill(
                  child: Listener(
                    onPointerDown: (event) {
                      widget.onPointerDown?.call(event.position);
                      // 确保点击时获取焦点，否则 EditableText 不会响应输入
                      if (!_focusNode.hasFocus) {
                        _focusNode.requestFocus();
                      }
                      // 将点击坐标转换为文本位置并移动光标
                      _handleTap(event.localPosition);
                    },
                    child: EditableText(
                      controller: _proxyController,
                      focusNode: _focusNode,
                      style: TextStyle(
                        color: Colors.transparent,
                        fontSize: widget.style.fontSize,
                        height: widget.style.height,
                        fontFamily: widget.style.fontFamily,
                        fontFamilyFallback: widget.style.fontFamilyFallback,
                      ),
                      cursorColor: Colors.transparent,
                      backgroundCursorColor: Colors.transparent,
                      selectionColor: Colors.transparent,
                      cursorOpacityAnimates: false,
                      scrollPadding: EdgeInsets.zero,
                      maxLines: null,
                      autocorrect: false,
                      enableSuggestions: false,
                      readOnly: _readOnly,
                      onChanged: (_) {}, // 变化由 controller listener 处理
                      onSelectionChanged: (_, __) {},
                    ),
                  ),
                ),
                // 层 2：CustomPaint（实际渲染）
                // IgnorePointer 确保点击事件穿透到下层 EditableText
                Positioned.fill(
                  child: IgnorePointer(
                    child: CustomPaint(
                      painter: CideEditorPainter(
                        document: widget.document,
                        scrollOffset: _scrollController.hasClients
                            ? _scrollController.offset
                            : 0.0,
                        viewportHeight: _viewportSize.height,
                        lineHeight: widget.style.fontSize != null
                            ? widget.style.fontSize! * 1.5
                            : 21.0,
                        textStyle: widget.style,
                        layers: widget.layers,
                      ),
                      size: Size.infinite,
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  void _handleTap(Offset position) {
    final lineHeight = widget.style.fontSize != null
        ? widget.style.fontSize! * 1.5
        : 21.0;

    // 注意：event.localPosition 是相对于 Listener（即 SingleChildScrollView 的 child）
    // 的坐标，已经自然包含了滚动偏移，不需要再加 scrollOffset。
    int line = (position.dy / lineHeight).floor().clamp(0, widget.document.lineCount - 1);

    // 获取该行文本
    final lineText = widget.document.lineText(line);

    // 使用 TextPainter 计算列号
    final textPainter = TextPainter(
      text: TextSpan(text: lineText, style: widget.style),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout();

    final textPosition = textPainter.getPositionForOffset(
      Offset(position.dx, lineHeight / 2),
    );
    int col = textPosition.offset.clamp(0, lineText.length);

    // 更新光标位置
    final newPos = DocPosition(line: line, col: col);
    widget.document.updateSelection(
      DocSelection(base: newPos, extent: newPos),
    );
    _syncToProxy();
  }
}
