part of '../custom_keyboard.dart';

extension _CustomKeyboardStateSymbols on _CustomKeyboardState {
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
                          onTap: () => _setSymbolCategory(index),
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
