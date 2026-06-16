import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../models/panel_item.dart';
import '../../providers/ide_provider.dart';
import '../../providers/theme_provider.dart';
import '../../widgets/algorithm_tab.dart';
import '../../widgets/array_vis_tab.dart';
import '../../widgets/breakpoints_tab.dart';
import '../../widgets/callstack_tab.dart';
import '../../widgets/diagnostics_tab.dart';
import '../../widgets/draggable_panel_tab.dart';
import '../../widgets/height_resizable_panel.dart';
import '../../widgets/intent_inference_panel.dart';
import '../../widgets/knowledge_card_tab.dart';
import '../../widgets/linked_list_vis_tab.dart';
import '../../widgets/memory_tab.dart';
import '../../widgets/output_tab.dart';
import '../../widgets/panel_drag_data.dart';
import '../../widgets/pointer_vis_tab.dart';
import '../../widgets/progress_tab.dart';
import '../../widgets/tree_vis_tab.dart';
import '../../widgets/var_history_tab.dart';
import '../../widgets/variables_tab.dart';
import '../../widgets/watch_tab.dart';

/// IDE 底部可伸缩面板组件。
///
/// 负责渲染底部 Tab 栏、拖拽交换、横向滑动手势切换，以及各面板内容。
/// 通过 [animation] 在键盘弹出时平滑收起。
class IdeBottomPanel extends ConsumerWidget {
  final Animation<double> animation;
  final TextEditingController inputController;
  final void Function(int line) onScrollToLine;
  final void Function(String source) onUpdateSource;

  const IdeBottomPanel({
    super.key,
    required this.animation,
    required this.inputController,
    required this.onScrollToLine,
    required this.onUpdateSource,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(ideProvider);
    final notifier = ref.read(ideProvider.notifier);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;
    final panelBg = isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff);

    final panelId = state.bottomSlots.isEmpty
        ? null
        : state.bottomSlots[state.bottomActiveIndex.clamp(
            0,
            state.bottomSlots.length - 1,
          )];

    return SizeTransition(
      sizeFactor: animation,
      axisAlignment: 1,
      child: HeightResizablePanel(
        height: state.bottomHeight,
        onHeightChanged: notifier.setBottomHeight,
        child: Container(
          decoration: BoxDecoration(
            color: panelBg,
            border: Border(
              top: BorderSide(
                color: Theme.of(context).dividerColor.withValues(alpha: 0.2),
              ),
            ),
          ),
          child: Column(
            children: [
              // Tab 栏（可拖拽交换）
              Container(
                height: 36,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                child: Row(
                  children: [
                    ...List.generate(state.bottomSlots.length, (index) {
                      final id = state.bottomSlots[index];
                      final item = PanelItem.fromId(id);
                      if (item == null) return const SizedBox.shrink();
                      final isActive = state.bottomActiveIndex == index;
                      return Expanded(
                        child: DraggablePanelTab(
                          item: item,
                          isActive: isActive,
                          badge: _getBadgeForPanel(id, state),
                          onTap: () => notifier.selectBottomTab(index),
                          data: PanelDragData(
                            panelId: id,
                            fromLocation: PanelLocation.bottom,
                            fromIndex: index,
                          ),
                          onAccept: (dragData) {
                            if (dragData.fromLocation == PanelLocation.bottom) {
                              // 同区：底部 Tab 之间交换位置
                              notifier.swapBottomPanels(
                                index,
                                dragData.fromIndex,
                              );
                            } else {
                              // 跨区域：悬浮球 → 底部，与当前位置交换
                              notifier.swapFloatingWithBottomItem(
                                dragData.panelId,
                                index,
                              );
                            }
                          },
                        ),
                      );
                    }),
                  ],
                ),
              ),
              // 内容区域（支持水平滑动切换标签）
              Expanded(
                child: GestureDetector(
                  onHorizontalDragEnd: (details) {
                    const threshold = 300.0;
                    final dx = details.velocity.pixelsPerSecond.dx;
                    if (dx > threshold && state.bottomActiveIndex > 0) {
                      notifier.selectBottomTab(state.bottomActiveIndex - 1);
                    } else if (dx < -threshold &&
                        state.bottomActiveIndex < state.bottomSlots.length - 1) {
                      notifier.selectBottomTab(state.bottomActiveIndex + 1);
                    }
                  },
                  child: panelId == null
                      ? const SizedBox.shrink()
                      : PanelContent(
                          panelId: panelId,
                          state: state,
                          notifier: notifier,
                          isDark: isDark,
                          inputController: inputController,
                          onScrollToLine: onScrollToLine,
                          onUpdateSource: onUpdateSource,
                        ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String? _getBadgeForPanel(String panelId, IdeState state) {
    switch (panelId) {
      case 'diagnostics':
        return state.diagnostics.isNotEmpty
            ? '${state.diagnostics.length}'
            : null;
      case 'algorithm':
        return state.algorithmMatches.isNotEmpty
            ? '${state.algorithmMatches.length}'
            : null;
      case 'breakpoints':
        return state.breakpoints.isNotEmpty
            ? '${state.breakpoints.length}'
            : null;
      default:
        return null;
    }
  }
}

/// 根据面板 ID 构建对应内容。
///
/// 同时供 [IdeBottomPanel] 与悬浮球弹窗使用，避免内容构建逻辑重复。
class PanelContent extends StatelessWidget {
  final String panelId;
  final IdeState state;
  final IdeNotifier notifier;
  final bool isDark;
  final TextEditingController? inputController;
  final void Function(int line)? onScrollToLine;
  final void Function(String source)? onUpdateSource;

  const PanelContent({
    super.key,
    required this.panelId,
    required this.state,
    required this.notifier,
    required this.isDark,
    this.inputController,
    this.onScrollToLine,
    this.onUpdateSource,
  });

  @override
  Widget build(BuildContext context) {
    switch (panelId) {
      case 'output':
        return OutputTab(
          state: state,
          notifier: notifier,
          isDark: isDark,
          inputController: inputController!,
        );
      case 'diagnostics':
        return DiagnosticsTab(
          state: state,
          notifier: notifier,
          isDark: isDark,
          onScrollToLine: onScrollToLine!,
          onUpdateSource: onUpdateSource!,
        );
      case 'algorithm':
        return AlgorithmTab(matches: state.algorithmMatches, isDark: isDark);
      case 'intent':
        return IntentInferencePanel(scores: state.intentScores);
      case 'knowledge':
        return KnowledgeCardTab(cards: state.knowledgeCards, isDark: isDark);
      case 'pointer':
        return PointerVisTab(isDark: isDark);
      case 'arrayVis':
        return ArrayVisTab(isDark: isDark);
      case 'linkedListVis':
        return LinkedListVisTab(isDark: isDark);
      case 'treeVis':
        return TreeVisTab(isDark: isDark);
      case 'memory':
        return MemoryTab(isDark: isDark);
      case 'variables':
        return VariablesTab(isDark: isDark);
      case 'watch':
        return WatchTab(watchExpressions: state.watchExpressions, isDark: isDark);
      case 'callstack':
        return CallstackTab(isDark: isDark, onScrollToLine: onScrollToLine);
      case 'progress':
        return ProgressTab(state: state);
      case 'varHistory':
        return VarHistoryTab(isDark: isDark);
      case 'breakpoints':
        return BreakpointsTab(
          state: state,
          isDark: isDark,
          onScrollToLine: onScrollToLine!,
        );
      default:
        return const SizedBox.shrink();
    }
  }
}
