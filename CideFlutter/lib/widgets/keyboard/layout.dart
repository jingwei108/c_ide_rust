part of '../custom_keyboard.dart';


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

  void _setSymbolCategory(int index) {
    setState(() => _symbolCategoryIndex = index);
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

}
