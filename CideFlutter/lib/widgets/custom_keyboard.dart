import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// 自定义虚拟键盘，用于替代系统键盘。
///
/// 包含：
/// - 快捷符号栏（与键盘同步弹出，零延迟）
/// - QWERTY 字母区
/// - 功能键：Shift、Backspace、Tab、Space、Enter
/// - 中/英切换：临时调用系统键盘输入中文
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

class _CustomKeyboardState extends State<CustomKeyboard> {
  bool _isUpperCase = false;

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

  // 快捷符号
  final List<({String label, VoidCallback onTap})> _symbols = [
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
    HapticFeedback.lightImpact();
    widget.onInsertText(key);
    if (_isUpperCase) {
      setState(() => _isUpperCase = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final keyBg = isDark ? const Color(0xff2c2c2e) : const Color(0xffe1e1e6);
    final keyTextColor = isDark ? Colors.white : Colors.black;
    final specialKeyBg = isDark ? const Color(0xff3a3a3c) : const Color(0xffd1d1d6);
    final keyboardBg = isDark ? const Color(0xff1c1c1e) : const Color(0xffd1d1d6);

    return FocusScope(
      canRequestFocus: false,
      child: Container(
        color: keyboardBg,
        child: SafeArea(
          top: false,
          child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // ========== 快捷符号栏 ==========
            Container(
              height: 40,
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(
                    color: isDark ? Colors.white12 : Colors.black12,
                  ),
                ),
              ),
              child: ListView.separated(
                scrollDirection: Axis.horizontal,
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
                itemCount: _symbols.length,
                separatorBuilder: (_, __) => const SizedBox(width: 4),
                itemBuilder: (context, index) {
                  final s = _symbols[index];
                  return _KeyButton(
                    label: s.label,
                    onTap: () => _onSymbolTap(s.label),
                    backgroundColor: specialKeyBg,
                    textColor: keyTextColor,
                    fontSize: 13,
                    padding: const EdgeInsets.symmetric(horizontal: 10),
                  );
                },
              ),
            ),

            // ========== 第一行字母 ==========
            _buildKeyRow(_keys[0], keyBg, keyTextColor),

            // ========== 第二行字母 ==========
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildKeyRow(_keys[1], keyBg, keyTextColor),
            ),

            // ========== 第三行字母 + Shift + Backspace ==========
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(
                children: [
                  // Shift
                  Expanded(
                    flex: 2,
                    child: _KeyButton(
                      label: '↑',
                      onTap: () {
                        HapticFeedback.lightImpact();
                        setState(() => _isUpperCase = !_isUpperCase);
                      },
                      backgroundColor: _isUpperCase
                          ? Colors.blueAccent
                          : specialKeyBg,
                      textColor: _isUpperCase ? Colors.white : keyTextColor,
                    ),
                  ),
                  const SizedBox(width: 4),
                  // 字母
                  ..._keys[2].map((k) => Expanded(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 2),
                          child: _KeyButton(
                            label: k,
                            onTap: () => _onKeyTap(k),
                            backgroundColor: keyBg,
                            textColor: keyTextColor,
                          ),
                        ),
                      )),
                  const SizedBox(width: 4),
                  // Backspace
                  Expanded(
                    flex: 2,
                    child: _KeyButton(
                      label: '⌫',
                      onTap: () {
                        HapticFeedback.lightImpact();
                        widget.onBackspace();
                      },
                      backgroundColor: specialKeyBg,
                      textColor: keyTextColor,
                    ),
                  ),
                ],
              ),
            ),

            // ========== 底部功能栏 ==========
            Padding(
              padding: const EdgeInsets.fromLTRB(8, 4, 8, 8),
              child: Row(
                children: [
                  // 中/英 切换
                  Expanded(
                    flex: 2,
                    child: _KeyButton(
                      label: widget.isSystemKeyboardActive ? '英' : '中',
                      onTap: () {
                        HapticFeedback.lightImpact();
                        widget.onToggleSystemKeyboard?.call();
                      },
                      backgroundColor: widget.isSystemKeyboardActive
                          ? Colors.blueAccent
                          : specialKeyBg,
                      textColor: widget.isSystemKeyboardActive
                          ? Colors.white
                          : keyTextColor,
                    ),
                  ),
                  const SizedBox(width: 4),
                  // Tab
                  Expanded(
                    flex: 2,
                    child: _KeyButton(
                      label: 'Tab',
                      onTap: () {
                        HapticFeedback.lightImpact();
                        widget.onTab();
                      },
                      backgroundColor: specialKeyBg,
                      textColor: keyTextColor,
                    ),
                  ),
                  const SizedBox(width: 4),
                  // Space（长条，显示 ␣）
                  Expanded(
                    flex: 5,
                    child: GestureDetector(
                      onTap: () {
                        HapticFeedback.lightImpact();
                        widget.onInsertText(' ');
                      },
                      child: Container(
                        height: 42,
                        decoration: BoxDecoration(
                          color: keyBg,
                          borderRadius: BorderRadius.circular(6),
                        ),
                        alignment: Alignment.center,
                        child: const Text(
                          '␣',
                          style: TextStyle(
                            fontSize: 18,
                            color: Colors.grey,
                            fontFamily: 'monospace',
                          ),
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 4),
                  // Enter
                  Expanded(
                    flex: 2,
                    child: _KeyButton(
                      label: '↵',
                      onTap: () {
                        HapticFeedback.lightImpact();
                        widget.onEnter();
                      },
                      backgroundColor: specialKeyBg,
                      textColor: keyTextColor,
                    ),
                  ),
                  const SizedBox(width: 4),
                  // 完成 / 收起键盘
                  Expanded(
                    flex: 2,
                    child: _KeyButton(
                      label: '完成',
                      onTap: () {
                        HapticFeedback.lightImpact();
                        widget.onDone?.call();
                      },
                      backgroundColor: Colors.blueAccent,
                      textColor: Colors.white,
                      fontSize: 14,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    ),
  );
  }

  Widget _buildKeyRow(
    List<String> keys,
    Color keyBg,
    Color keyTextColor,
  ) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(4, 4, 4, 0),
      child: Row(
        children: keys.map((k) {
          return Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 2),
              child: _KeyButton(
                label: k,
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
}

/// 单个按键按钮
class _KeyButton extends StatelessWidget {
  final String label;
  final VoidCallback onTap;
  final Color backgroundColor;
  final Color textColor;
  final double fontSize;
  final EdgeInsetsGeometry padding;

  const _KeyButton({
    required this.label,
    required this.onTap,
    required this.backgroundColor,
    required this.textColor,
    this.fontSize = 16,
    this.padding = EdgeInsets.zero,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      behavior: HitTestBehavior.opaque,
      child: Container(
        height: 42,
        padding: padding,
        decoration: BoxDecoration(
          color: backgroundColor,
          borderRadius: BorderRadius.circular(6),
        ),
        alignment: Alignment.center,
        child: Text(
          label,
          style: TextStyle(
            fontSize: fontSize,
            color: textColor,
            fontWeight: FontWeight.w500,
          ),
        ),
      ),
    );
  }
}
