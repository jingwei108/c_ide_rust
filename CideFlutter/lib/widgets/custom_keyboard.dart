import 'dart:async';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// 自定义虚拟键盘，用于替代系统键盘。
///
/// 三种模式：
/// - 字母模式（ABC）：QWERTY + 精简快捷符号栏
/// - 数字模式（123）：九宫格数字 + 侧边符号
/// - 符号模式（符）：C 语言符号网格（可滑动）
class CustomKeyboard extends StatefulWidget {
  final VoidCallback? onClose;
  final void Function(String text) onInsertText;
  final void Function(String left, String right) onInsertPair;
  final void Function(int delta) onMoveCursor;
  final VoidCallback onBackspace;
  final VoidCallback onEnter;
  final VoidCallback onTab;
  final VoidCallback? onUndo;
  final VoidCallback? onRedo;
  final VoidCallback? onDone;
  final VoidCallback? onToggleSystemKeyboard;
  final bool isSystemKeyboardActive;

  const CustomKeyboard({
    super.key,
    this.onClose,
    required this.onInsertText,
    required this.onInsertPair,
    required this.onMoveCursor,
    required this.onBackspace,
    required this.onEnter,
    required this.onTab,
    this.onUndo,
    this.onRedo,
    this.onDone,
    this.onToggleSystemKeyboard,
    this.isSystemKeyboardActive = false,
  });

  @override
  State<CustomKeyboard> createState() => _CustomKeyboardState();
}

enum _KeyboardMode { letters, numbers, symbols }

class _CustomKeyboardState extends State<CustomKeyboard> {
  bool _isUpperCase = false;
  _KeyboardMode _mode = _KeyboardMode.letters;
  int _symbolCategoryIndex = 0;
  double _spaceDragAccum = 0;

  // QWERTY 布局（小写）
  final List<List<String>> _keysLower = [
    ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
    ['z', 'x', 'c', 'v', 'b', 'n', 'm'],
  ];

  // QWERTY 布局（大写）
  final List<List<String>> _keysUpper = [
    ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
    ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
    ['Z', 'X', 'C', 'V', 'B', 'N', 'M'],
  ];

  List<List<String>> get _keys => _isUpperCase ? _keysUpper : _keysLower;

  // 字母模式下的精简快捷符号栏（最常用代码符号）
  final List<({String label, VoidCallback onTap})> _letterSymbols = [
    (label: '{ }', onTap: () {}),
    (label: '( )', onTap: () {}),
    (label: '[ ]', onTap: () {}),
    (label: '" "', onTap: () {}),
    (label: "' '", onTap: () {}),
    (label: ';', onTap: () {}),
    (label: '#', onTap: () {}),
    (label: '->', onTap: () {}),
    (label: '&', onTap: () {}),
    (label: '*', onTap: () {}),
    (label: '=', onTap: () {}),
    (label: '==', onTap: () {}),
    (label: '!=', onTap: () {}),
    (label: '<', onTap: () {}),
    (label: '>', onTap: () {}),
    (label: '+', onTap: () {}),
    (label: '-', onTap: () {}),
    (label: '/', onTap: () {}),
    (label: '%', onTap: () {}),
    (label: '&&', onTap: () {}),
    (label: '||', onTap: () {}),
    (label: '!', onTap: () {}),
    (label: '|', onTap: () {}),
    (label: '^', onTap: () {}),
    (label: '~', onTap: () {}),
    (label: ',', onTap: () {}),
    (label: '.', onTap: () {}),
    (label: '_', onTap: () {}),
    (label: ':', onTap: () {}),
  ];

  // 数字模式：九宫格 + 侧边符号
  // 布局（文字描述）：
  // % | 1 | 2 | 3 | 删除
  // / | 4 | 5 | 6 | Tab
  // - | 7 | 8 | 9 | 回车
  // + |
  final List<String> _numPadLeft = ['%', '/', '-', '+'];
  final List<List<String>> _numPadGrid = [
    ['1', '2', '3'],
    ['4', '5', '6'],
    ['7', '8', '9'],
  ];
  // 右侧功能栏（删除 / Tab / 回车）—— 直接在 build 中硬编码布局

  // 符号模式分类
  final List<({String name, List<String> symbols})> _symbolCategories = [
    (
      name: '常用',
      symbols: [
        '{', '}', '(', ')', '[', ']',
        '"', "'", ';', '#', ',', '.',
        '_', ':', '\\', '/',
      ],
    ),
    (
      name: '运算符',
      symbols: [
        '+', '-', '*', '/', '%',
        '++', '--', '->', '.',
        '&&', '||', '!', '&', '|', '^', '~',
      ],
    ),
    (
      name: '比较',
      symbols: [
        '=', '==', '!=',
        '<', '>', '<=', '>=',
      ],
    ),
    (
      name: '位移',
      symbols: [
        '<<', '>>',
        '&', '|', '^', '~',
      ],
    ),
    (
      name: '其他',
      symbols: [
        '@', '#', '\$', '?',
        '...', '::', '`',
        'NULL', 'sizeof',
      ],
    ),
  ];

  void _onSymbolTap(String label) {
    HapticFeedback.lightImpact();
    switch (label) {
      case '{ }':
        widget.onInsertPair('{', '}');
      case '( )':
        widget.onInsertPair('(', ')');
      case '[ ]':
        widget.onInsertPair('[', ']');
      case '" "':
        widget.onInsertPair('"', '"');
      case "' '":
        widget.onInsertPair("'", "'");
      default:
        widget.onInsertText(label);
    }
  }

  void _onKeyTap(String key) {
    widget.onInsertText(key);
    if (_isUpperCase) {
      setState(() => _isUpperCase = false);
    }
  }

  void _handleSpaceDrag(double delta) {
    _spaceDragAccum += delta;
    const threshold = 12.0;
    while (_spaceDragAccum >= threshold) {
      _spaceDragAccum -= threshold;
      widget.onMoveCursor(1);
    }
    while (_spaceDragAccum <= -threshold) {
      _spaceDragAccum += threshold;
      widget.onMoveCursor(-1);
    }
  }

  void _setMode(_KeyboardMode mode) {
    setState(() => _mode = mode);
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final keyBg = isDark ? const Color(0xff3a3a3c) : const Color(0xffe1e1e6);
    final keyTextColor = isDark ? Colors.white : Colors.black;
    final specialKeyBg = isDark ? const Color(0xff4a4a4c) : const Color(0xffd1d1d6);
    final keyboardBg = isDark ? const Color(0xff1c1c1e) : const Color(0xffd1d1d6);

    return FocusScope(
      canRequestFocus: false,
      child: Container(
        color: keyboardBg,
        child: SafeArea(
          top: false,
          child: AnimatedSize(
            duration: const Duration(milliseconds: 200),
            curve: Curves.easeInOut,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // ========== 字母模式 ==========
                if (_mode == _KeyboardMode.letters) ...[
                  _buildSymbolBar(specialKeyBg, keyTextColor),
                  _buildKeyRowWithNumbers(_keys[0], keyBg, keyTextColor, null),
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 18),
                    child: _buildKeyRowWithNumbers(_keys[1], keyBg, keyTextColor, null),
                  ),
                  Padding(
                    padding: const EdgeInsets.fromLTRB(10, 6, 10, 0),
                    child: Row(
                      children: [
                        // Shift
                        Expanded(
                          flex: 2,
                          child: _KeyButton(
                            label: '⇧',
                            onTap: () => setState(() => _isUpperCase = !_isUpperCase),
                            backgroundColor: _isUpperCase ? Colors.blueAccent : specialKeyBg,
                            textColor: _isUpperCase ? Colors.white : keyTextColor,
                          ),
                        ),
                        const SizedBox(width: 6),
                        ..._keys[2].map((k) => Expanded(
                              child: Padding(
                                padding: const EdgeInsets.symmetric(horizontal: 3),
                                child: _KeyButton(
                                  label: k,
                                  onTap: () => _onKeyTap(k),
                                  backgroundColor: keyBg,
                                  textColor: keyTextColor,
                                ),
                              ),
                            )),
                        const SizedBox(width: 6),
                        // Backspace
                        Expanded(
                          flex: 2,
                          child: _KeyButton(
                            label: '⌫',
                            onTap: widget.onBackspace,
                            repeatOnLongPress: true,
                            backgroundColor: specialKeyBg,
                            textColor: keyTextColor,
                          ),
                        ),
                      ],
                    ),
                  ),
                  _buildLetterBottomRow(specialKeyBg, keyTextColor, keyBg),
                ],

                // ========== 数字模式 ==========
                if (_mode == _KeyboardMode.numbers) ...[
                  _buildNumPad(keyBg, keyTextColor, specialKeyBg),
                  _buildNumberBottomRow(specialKeyBg, keyTextColor, keyBg),
                ],

                // ========== 符号模式 ==========
                if (_mode == _KeyboardMode.symbols) ...[
                  _buildSymbolGrid(keyBg, keyTextColor, specialKeyBg),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }

  // ========== 字母模式组件 ==========

  Widget _buildSymbolBar(Color specialKeyBg, Color keyTextColor) {
    return Container(
      height: 36,
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).brightness == Brightness.dark
                ? Colors.white.withValues(alpha: 0.1)
                : Colors.black.withValues(alpha: 0.1),
          ),
        ),
      ),
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
        itemCount: _letterSymbols.length,
        separatorBuilder: (_, __) => const SizedBox(width: 4),
        itemBuilder: (context, index) {
          final s = _letterSymbols[index];
          return _KeyButton(
            label: s.label,
            onTap: () => _onSymbolTap(s.label),
            backgroundColor: specialKeyBg,
            textColor: keyTextColor,
            fontSize: 12,
            padding: const EdgeInsets.symmetric(horizontal: 10),
          );
        },
      ),
    );
  }

  Widget _buildKeyRowWithNumbers(
    List<String> keys,
    Color keyBg,
    Color keyTextColor,
    List<String>? numbers,
  ) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(4, 6, 4, 0),
      child: Row(
        children: keys.asMap().entries.map((entry) {
          final i = entry.key;
          final k = entry.value;
          return Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 3),
              child: _LetterKey(
                letter: k,
                number: numbers != null && i < numbers.length ? numbers[i] : null,
                onTap: () => _onKeyTap(k),
                backgroundColor: keyBg,
                textColor: keyTextColor,
              ),
            ),
          );
        }).toList(),
      ),
    );
  }

  Widget _buildLetterBottomRow(Color specialKeyBg, Color keyTextColor, Color keyBg) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(6, 6, 6, 10),
      child: Row(
        children: [
          // 123
          Expanded(
            flex: 2,
            child: _KeyButton(
              label: '123',
              onTap: () => _setMode(_KeyboardMode.numbers),
              backgroundColor: specialKeyBg,
              textColor: keyTextColor,
              fontSize: 16,
            ),
          ),
          const SizedBox(width: 6),
          // 中/英 切换
          Expanded(
            flex: 2,
            child: _KeyButton(
              label: widget.isSystemKeyboardActive ? '英' : '中',
              onTap: () => widget.onToggleSystemKeyboard?.call(),
              backgroundColor: widget.isSystemKeyboardActive ? Colors.blueAccent : specialKeyBg,
              textColor: widget.isSystemKeyboardActive ? Colors.white : keyTextColor,
              fontSize: 16,
            ),
          ),
          const SizedBox(width: 6),
          // 符
          Expanded(
            flex: 2,
            child: _KeyButton(
              label: '符',
              onTap: () => _setMode(_KeyboardMode.symbols),
              backgroundColor: specialKeyBg,
              textColor: keyTextColor,
              fontSize: 16,
            ),
          ),
          const SizedBox(width: 6),
          // Space
          Expanded(
            flex: 5,
            child: _KeyButton(
              label: '␣',
              onTap: () => widget.onInsertText(' '),
              onHorizontalDragUpdate: (details) => _handleSpaceDrag(details.delta.dx),
              backgroundColor: keyBg,
              textColor: Theme.of(context).brightness == Brightness.dark ? Colors.grey : Colors.black54,
              fontSize: 18,
            ),
          ),
          const SizedBox(width: 6),
          // ↵ Enter
          Expanded(
            flex: 2,
            child: _KeyButton(
              label: '↵',
              onTap: widget.onEnter,
              backgroundColor: specialKeyBg,
              textColor: keyTextColor,
              fontSize: 20,
            ),
          ),
          const SizedBox(width: 6),
          // Tab
          Expanded(
            flex: 2,
            child: _KeyButton(
              label: 'Tab',
              onTap: widget.onTab,
              backgroundColor: specialKeyBg,
              textColor: keyTextColor,
              fontSize: 12,
            ),
          ),
          const SizedBox(width: 6),
          // 完成
          Expanded(
            flex: 2,
            child: _KeyButton(
              label: '完成',
              onTap: () => widget.onDone?.call(),
              backgroundColor: Colors.blueAccent,
              textColor: Colors.white,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }

  // ========== 数字模式组件 ==========

  Widget _buildNumPad(Color keyBg, Color keyTextColor, Color specialKeyBg) {
    return SizedBox(
      height: 230,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(6, 6, 6, 0),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // 左侧符号栏（% / - +）— 4个键等分总高度
            SizedBox(
              width: 48,
              child: Column(
                children: _numPadLeft.map((s) {
                  return Expanded(
                    child: Padding(
                      padding: const EdgeInsets.only(bottom: 6),
                      child: _KeyButton(
                        label: s,
                        onTap: () => widget.onInsertText(s),
                        backgroundColor: specialKeyBg,
                        textColor: keyTextColor,
                        fontSize: 16,
                        height: null,
                      ),
                    ),
                  );
                }).toList(),
              ),
            ),
            const SizedBox(width: 6),
            // 中间九宫格 — 3行等分总高度
            Expanded(
              child: Column(
                children: _numPadGrid.map((row) {
                  return Expanded(
                    child: Padding(
                      padding: const EdgeInsets.only(bottom: 6),
                      child: Row(
                        children: row.map((k) {
                          return Expanded(
                            child: Padding(
                              padding: const EdgeInsets.symmetric(horizontal: 3),
                              child: _KeyButton(
                                label: k,
                                onTap: () => widget.onInsertText(k),
                                backgroundColor: keyBg,
                                textColor: keyTextColor,
                                fontSize: 20,
                                height: null,
                              ),
                            ),
                          );
                        }).toList(),
                      ),
                    ),
                  );
                }).toList(),
              ),
            ),
            const SizedBox(width: 6),
            // 右侧功能栏（删除 / Tab / 回车）— 3个键等分总高度
            SizedBox(
              width: 56,
              child: Column(
                children: [
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.only(bottom: 6),
                      child: _KeyButton(
                        label: '⌫',
                        onTap: widget.onBackspace,
                        repeatOnLongPress: true,
                        backgroundColor: specialKeyBg,
                        textColor: keyTextColor,
                        fontSize: 18,
                        height: null,
                      ),
                    ),
                  ),
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.only(bottom: 6),
                      child: _KeyButton(
                        label: 'Tab',
                        onTap: widget.onTab,
                        backgroundColor: specialKeyBg,
                        textColor: keyTextColor,
                        fontSize: 12,
                        height: null,
                      ),
                    ),
                  ),
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.only(bottom: 6),
                      child: _KeyButton(
                        label: '↵',
                        onTap: widget.onEnter,
                        backgroundColor: specialKeyBg,
                        textColor: keyTextColor,
                        fontSize: 18,
                        height: null,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNumberBottomRow(Color specialKeyBg, Color keyTextColor, Color keyBg) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(6, 4, 6, 10),
      child: Row(
        children: [
          // 左侧栏占位 — 符
          SizedBox(
            width: 48,
            child: _KeyButton(
              label: '符',
              onTap: () => _setMode(_KeyboardMode.symbols),
              backgroundColor: specialKeyBg,
              textColor: keyTextColor,
              fontSize: 14,
            ),
          ),
          const SizedBox(width: 6),
          // 中间区域：返回 | 0 | 空格（0对准中间列）
          Expanded(
            child: Row(
              children: [
                Expanded(
                  child: _KeyButton(
                    label: '返回',
                    onTap: () => _setMode(_KeyboardMode.letters),
                    backgroundColor: specialKeyBg,
                    textColor: keyTextColor,
                    fontSize: 14,
                  ),
                ),
                const SizedBox(width: 6),
                Expanded(
                  child: _KeyButton(
                    label: '0',
                    onTap: () => widget.onInsertText('0'),
                    backgroundColor: keyBg,
                    textColor: keyTextColor,
                    fontSize: 20,
                  ),
                ),
                const SizedBox(width: 6),
                Expanded(
                  child: _KeyButton(
                    label: '␣',
                    onTap: () => widget.onInsertText(' '),
                    backgroundColor: keyBg,
                    textColor: Theme.of(context).brightness == Brightness.dark ? Colors.grey : Colors.black54,
                    fontSize: 18,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 6),
          // 右侧栏占位 — 完成
          SizedBox(
            width: 56,
            child: _KeyButton(
              label: '完成',
              onTap: () => widget.onDone?.call(),
              backgroundColor: Colors.blueAccent,
              textColor: Colors.white,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }

  // ========== 符号模式组件 ==========

  Widget _buildSymbolGrid(Color keyBg, Color keyTextColor, Color specialKeyBg) {
    final category = _symbolCategories[_symbolCategoryIndex];
    final symbols = category.symbols;
    const columns = 4;

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 主体：左侧菜单 + 右侧可滑动符号格
        SizedBox(
          height: 220,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // 左侧分类菜单
              Container(
                width: 64,
                decoration: BoxDecoration(
                  color: Theme.of(context).brightness == Brightness.dark
                      ? Colors.black12
                      : Colors.white10,
                  border: Border(
                    right: BorderSide(
                      color: Theme.of(context).brightness == Brightness.dark
                          ? Colors.white10
                          : Colors.black.withValues(alpha: 0.1),
                    ),
                  ),
                ),
                child: ListView.builder(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  itemCount: _symbolCategories.length + 1, // +1 for 返回
                  itemBuilder: (context, index) {
                    if (index == _symbolCategories.length) {
                      return Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                        child: _KeyButton(
                          label: '返回',
                          onTap: () => _setMode(_KeyboardMode.letters),
                          backgroundColor: specialKeyBg,
                          textColor: keyTextColor,
                          fontSize: 12,
                          height: 40,
                        ),
                      );
                    }
                    final cat = _symbolCategories[index];
                    final isActive = index == _symbolCategoryIndex;
                    return Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                      child: _KeyButton(
                        label: cat.name,
                        onTap: () => setState(() => _symbolCategoryIndex = index),
                        backgroundColor: isActive ? Colors.blueAccent : specialKeyBg,
                        textColor: isActive ? Colors.white : keyTextColor,
                        fontSize: 12,
                        height: 40,
                      ),
                    );
                  },
                ),
              ),
              // 右侧符号网格（可上下滑动）
              Expanded(
                child: Scrollbar(
                  child: GridView.builder(
                    padding: const EdgeInsets.all(6),
                    gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                      crossAxisCount: columns,
                      mainAxisSpacing: 6,
                      crossAxisSpacing: 6,
                      childAspectRatio: 1.6,
                    ),
                    itemCount: symbols.length,
                    itemBuilder: (context, index) {
                      final sym = symbols[index];
                      return _KeyButton(
                        label: sym,
                        onTap: () => _onSymbolTap(sym),
                        backgroundColor: keyBg,
                        textColor: keyTextColor,
                        fontSize: 14,
                        height: 48,
                      );
                    },
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

// ========== 子组件 ==========

/// 带顶部小数字提示的字母按键
class _LetterKey extends StatelessWidget {
  final String letter;
  final String? number;
  final VoidCallback onTap;
  final Color backgroundColor;
  final Color textColor;

  const _LetterKey({
    required this.letter,
    this.number,
    required this.onTap,
    required this.backgroundColor,
    required this.textColor,
  });

  @override
  Widget build(BuildContext context) {
    return _KeyButton(
      label: letter,
      onTap: onTap,
      backgroundColor: backgroundColor,
      textColor: textColor,
      fontSize: 18,
      topLabel: number,
      topLabelColor: textColor.withValues(alpha: 0.45),
    );
  }
}

/// 单个按键按钮 —— 带按压动画、触觉反馈、长按连发、水平拖拽
class _KeyButton extends StatefulWidget {
  final String label;
  final VoidCallback onTap;
  final GestureDragUpdateCallback? onHorizontalDragUpdate;
  final bool repeatOnLongPress;
  final Color backgroundColor;
  final Color textColor;
  final double fontSize;
  final EdgeInsetsGeometry padding;
  final double? height;
  final String? topLabel;
  final Color? topLabelColor;

  const _KeyButton({
    required this.label,
    required this.onTap,
    this.onHorizontalDragUpdate,
    this.repeatOnLongPress = false,
    required this.backgroundColor,
    required this.textColor,
    this.fontSize = 16,
    this.padding = EdgeInsets.zero,
    this.height = 48,
    this.topLabel,
    this.topLabelColor,
  });

  @override
  State<_KeyButton> createState() => _KeyButtonState();
}

class _KeyButtonState extends State<_KeyButton>
    with SingleTickerProviderStateMixin {
  bool _pressed = false;
  Timer? _longPressTimer;
  Timer? _repeatTimer;
  late final AnimationController _controller;
  late final Animation<double> _scaleAnim;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 80),
    );
    _scaleAnim = Tween<double>(begin: 1.0, end: 0.92).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOut),
    );
  }

  @override
  void dispose() {
    _cancelRepeat();
    _controller.dispose();
    super.dispose();
  }

  void _cancelRepeat() {
    _longPressTimer?.cancel();
    _longPressTimer = null;
    _repeatTimer?.cancel();
    _repeatTimer = null;
  }

  void _onTapDown(TapDownDetails details) {
    if (!mounted) return;
    setState(() => _pressed = true);
    _controller.forward();
    HapticFeedback.selectionClick();

    if (widget.repeatOnLongPress) {
      _longPressTimer = Timer(const Duration(milliseconds: 400), () {
        if (!mounted) return;
        widget.onTap();
        _repeatTimer = Timer.periodic(
          const Duration(milliseconds: 80),
          (_) {
            if (mounted) widget.onTap();
          },
        );
      });
    }
  }

  void _onTapUp(TapUpDetails details) {
    if (!mounted) return;
    final wasRepeating = _repeatTimer != null;
    _cancelRepeat();
    setState(() => _pressed = false);
    _controller.reverse();
    if (!wasRepeating) {
      widget.onTap();
    }
  }

  void _onTapCancel() {
    if (!mounted) return;
    _cancelRepeat();
    setState(() => _pressed = false);
    _controller.reverse();
  }

  void _onHorizontalDragStart(DragStartDetails details) {
    _cancelRepeat();
    if (_pressed) {
      setState(() => _pressed = false);
      _controller.reverse();
    }
  }

  void _onHorizontalDragEnd(DragEndDetails details) {
    if (_pressed) {
      setState(() => _pressed = false);
      _controller.reverse();
    }
  }

  Color get _bgColor {
    if (!_pressed) return widget.backgroundColor;
    final hsl = HSLColor.fromColor(widget.backgroundColor);
    return hsl
        .withLightness((hsl.lightness - 0.12).clamp(0.0, 1.0))
        .toColor();
  }

  bool get _isDark => Theme.of(context).brightness == Brightness.dark;

  List<BoxShadow> get _shadows {
    if (_pressed) return const [];
    return [
      BoxShadow(
        color: Colors.black.withValues(alpha: _isDark ? 0.35 : 0.12),
        blurRadius: 2,
        offset: const Offset(0, 1.5),
      ),
    ];
  }

  @override
  Widget build(BuildContext context) {
    final gestures = <Type, GestureRecognizerFactory>{};

    gestures[TapGestureRecognizer] =
        GestureRecognizerFactoryWithHandlers<TapGestureRecognizer>(
      () => TapGestureRecognizer(),
      (instance) {
        instance.onTapDown = _onTapDown;
        instance.onTapUp = _onTapUp;
        instance.onTapCancel = _onTapCancel;
      },
    );

    if (widget.onHorizontalDragUpdate != null) {
      gestures[HorizontalDragGestureRecognizer] =
          GestureRecognizerFactoryWithHandlers<HorizontalDragGestureRecognizer>(
        () => HorizontalDragGestureRecognizer(),
        (instance) {
          instance.onStart = _onHorizontalDragStart;
          instance.onUpdate = widget.onHorizontalDragUpdate;
          instance.onEnd = _onHorizontalDragEnd;
        },
      );
    }

    Widget content;
    if (widget.topLabel != null) {
      content = Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            widget.topLabel!,
            style: TextStyle(
              fontSize: widget.fontSize * 0.55,
              color: widget.topLabelColor ?? widget.textColor.withValues(alpha: 0.5),
              fontWeight: FontWeight.w500,
              height: 1.0,
            ),
          ),
          Text(
            widget.label,
            style: TextStyle(
              fontSize: widget.fontSize,
              color: widget.textColor,
              fontWeight: FontWeight.w500,
              height: 1.2,
            ),
          ),
        ],
      );
    } else {
      content = Text(
        widget.label,
        style: TextStyle(
          fontSize: widget.fontSize,
          color: widget.textColor,
          fontWeight: FontWeight.w500,
        ),
      );
    }

    return RawGestureDetector(
      gestures: gestures,
      behavior: HitTestBehavior.opaque,
      child: AnimatedBuilder(
        animation: _scaleAnim,
        builder: (context, child) {
          return Transform.scale(
            scale: _scaleAnim.value,
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 60),
              height: widget.height,
              padding: widget.padding,
              decoration: BoxDecoration(
                color: _bgColor,
                borderRadius: BorderRadius.circular(8),
                boxShadow: _shadows,
              ),
              alignment: Alignment.center,
              child: content,
            ),
          );
        },
      ),
    );
  }
}
