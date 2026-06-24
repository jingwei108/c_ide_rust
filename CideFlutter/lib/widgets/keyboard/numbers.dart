part of '../custom_keyboard.dart';

extension _CustomKeyboardStateNumbers on _CustomKeyboardState {
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

}
