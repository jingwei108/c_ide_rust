part of '../ide_notifier.dart';

/// 面板与布局管理：底部 Tab、悬浮球、面板交换、高亮线。
mixin PanelNotifierMixin on AutoDisposeNotifier<IdeState> {
  /// 选择底部 Tab
  void selectBottomTab(int index) {
    state = state.copyWith(bottomActiveIndex: index);
  }

  /// 选择悬浮球 Tab
  void selectFloatingTab(int index) {
    state = state.copyWith(floatingActiveIndex: index);
  }

  /// 设置底部面板高度
  void setBottomHeight(double height) {
    state = state.copyWith(bottomHeight: height.clamp(120, 500));
  }

  /// 切换悬浮球菜单展开/收起
  void toggleFloating() {
    state = state.copyWith(isFloatingOpen: !state.isFloatingOpen);
  }

  /// 关闭悬浮球菜单
  void closeFloating() {
    state = state.copyWith(isFloatingOpen: false);
  }

  /// 打开指定浮动面板弹窗
  void openFloatingPanel(String panelId) {
    state = state.copyWith(activeFloatingPanel: panelId, isFloatingOpen: false);
  }

  /// 关闭浮动面板弹窗
  void closeFloatingPanel() {
    state = state.copyWith(clearActiveFloatingPanel: true);
  }

  /// 将面板与底部区域交换（跨区域：悬浮球 → 底部）
  /// 保持两区元素总数不变
  void swapWithBottom(String panelId) {
    final currentFloating = List<String>.from(state.floatingSlots);
    final currentBottom = List<String>.from(state.bottomSlots);

    if (currentBottom.contains(panelId)) return;

    // 从悬浮球移除
    currentFloating.remove(panelId);

    // 如果底部已满，把底部最后一个挤出到悬浮球
    String? overflow;
    if (currentBottom.length >= 3) {
      overflow = currentBottom.removeLast();
      currentFloating.add(overflow);
    }

    currentBottom.add(panelId);

    state = state.copyWith(
      bottomSlots: currentBottom,
      floatingSlots: currentFloating,
      bottomActiveIndex: currentBottom
          .indexOf(panelId)
          .clamp(0, currentBottom.length - 1),
      floatingActiveIndex:
          overflow != null
              ? currentFloating
                  .indexOf(overflow)
                  .clamp(0, currentFloating.length - 1)
              : state.floatingActiveIndex,
      error: null,
    );
  }

  /// 将面板与悬浮球区域交换（跨区域：底部 → 悬浮球）
  /// 保持两区元素总数不变
  void swapWithFloating(String panelId) {
    final currentFloating = List<String>.from(state.floatingSlots);
    final currentBottom = List<String>.from(state.bottomSlots);

    if (currentFloating.contains(panelId)) return;

    // 从底部移除
    currentBottom.remove(panelId);

    // 如果悬浮球已满，把悬浮球最后一个挤出到底部
    String? overflow;
    if (currentFloating.length >= 8) {
      overflow = currentFloating.removeLast();
      currentBottom.add(overflow);
    }

    currentFloating.add(panelId);

    state = state.copyWith(
      bottomSlots: currentBottom,
      floatingSlots: currentFloating,
      floatingActiveIndex: currentFloating
          .indexOf(panelId)
          .clamp(0, currentFloating.length - 1),
      bottomActiveIndex:
          overflow != null
              ? currentBottom
                  .indexOf(overflow)
                  .clamp(0, currentBottom.length - 1)
              : state.bottomActiveIndex,
      error: null,
    );
  }

  /// 交换底部两个面板位置
  void swapBottomPanels(int indexA, int indexB) {
    final slots = List<String>.from(state.bottomSlots);
    if (indexA < 0 || indexA >= slots.length) return;
    if (indexB < 0 || indexB >= slots.length) return;
    final temp = slots[indexA];
    slots[indexA] = slots[indexB];
    slots[indexB] = temp;
    state = state.copyWith(bottomSlots: slots);
  }

  /// 交换悬浮球两个面板位置
  void swapFloatingPanels(int indexA, int indexB) {
    final slots = List<String>.from(state.floatingSlots);
    if (indexA < 0 || indexA >= slots.length) return;
    if (indexB < 0 || indexB >= slots.length) return;
    final temp = slots[indexA];
    slots[indexA] = slots[indexB];
    slots[indexB] = temp;
    state = state.copyWith(floatingSlots: slots);
  }

  /// 跨区域交换：底部指定面板 ↔ 悬浮球指定位置
  void swapBottomWithFloatingItem(String bottomPanelId, int floatingIndex) {
    final currentFloating = List<String>.from(state.floatingSlots);
    final currentBottom = List<String>.from(state.bottomSlots);

    final bottomIndex = currentBottom.indexOf(bottomPanelId);
    if (bottomIndex == -1) return;
    if (floatingIndex < 0 || floatingIndex >= currentFloating.length) return;

    final floatingPanelId = currentFloating[floatingIndex];

    currentBottom[bottomIndex] = floatingPanelId;
    currentFloating[floatingIndex] = bottomPanelId;

    state = state.copyWith(
      bottomSlots: currentBottom,
      floatingSlots: currentFloating,
      bottomActiveIndex: bottomIndex,
      floatingActiveIndex: floatingIndex,
      error: null,
    );
  }

  /// 跨区域交换：悬浮球指定面板 ↔ 底部指定位置
  void swapFloatingWithBottomItem(String floatingPanelId, int bottomIndex) {
    final currentFloating = List<String>.from(state.floatingSlots);
    final currentBottom = List<String>.from(state.bottomSlots);

    final floatingIndex = currentFloating.indexOf(floatingPanelId);
    if (floatingIndex == -1) return;
    if (bottomIndex < 0 || bottomIndex >= currentBottom.length) return;

    final bottomPanelId = currentBottom[bottomIndex];

    currentFloating[floatingIndex] = bottomPanelId;
    currentBottom[bottomIndex] = floatingPanelId;

    state = state.copyWith(
      bottomSlots: currentBottom,
      floatingSlots: currentFloating,
      bottomActiveIndex: bottomIndex,
      floatingActiveIndex: floatingIndex,
      error: null,
    );
  }

  /// 双击底部面板标题：删除并移到悬浮球
  void removeBottomPanel(int index) {
    final bottom = List<String>.from(state.bottomSlots);
    final floating = List<String>.from(state.floatingSlots);
    if (index < 0 || index >= bottom.length) return;

    final panelId = bottom.removeAt(index);
    if (!floating.contains(panelId)) {
      if (floating.length >= 8) {
        state = state.copyWith(error: '悬浮球承载已达上限（最多8个）');
        return;
      }
      floating.add(panelId);
    }

    state = state.copyWith(
      bottomSlots: bottom,
      floatingSlots: floating,
      bottomActiveIndex: state.bottomActiveIndex.clamp(
        0,
        (bottom.length - 1).clamp(0, 999),
      ),
    );
  }

  /// 双击悬浮球面板标题：删除并移到底部
  void removeFloatingPanel(int index) {
    final bottom = List<String>.from(state.bottomSlots);
    final floating = List<String>.from(state.floatingSlots);
    if (index < 0 || index >= floating.length) return;

    final panelId = floating.removeAt(index);
    if (!bottom.contains(panelId)) {
      if (bottom.length >= 3) {
        // 底部已满，把最后一个移到悬浮球
        final overflow = bottom.removeLast();
        if (!floating.contains(overflow)) {
          floating.add(overflow);
        }
      }
      bottom.add(panelId);
    }

    state = state.copyWith(
      bottomSlots: bottom,
      floatingSlots: floating,
      floatingActiveIndex: state.floatingActiveIndex.clamp(
        0,
        (floating.length - 1).clamp(0, 999),
      ),
    );
  }

  void highlightLine(int line) {
    state = state.copyWith(highlightedLine: line);
  }

  void clearHighlight() {
    state = state.copyWith(highlightedLine: 0);
  }
}
