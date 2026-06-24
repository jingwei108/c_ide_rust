part of '../custom_keyboard.dart';

extension _CustomKeyboardStateLetters on _CustomKeyboardState {
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

}
