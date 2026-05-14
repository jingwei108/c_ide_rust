import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/code_template.dart';
import '../models/ide_state.dart';
import '../models/panel_item.dart';
import '../providers/ide_provider.dart';
import '../widgets/draggable_panel_tab.dart';
import '../widgets/editor_panel.dart';
import '../widgets/height_resizable_panel.dart';
import '../widgets/intro_overlay.dart';
import '../widgets/panel_drag_data.dart';
import '../widgets/algorithm_tab.dart';
import '../widgets/array_vis_tab.dart';
import '../widgets/callstack_tab.dart';
import '../widgets/diagnostics_tab.dart';
import '../widgets/knowledge_card_tab.dart';
import '../widgets/memory_tab.dart';
import '../widgets/output_tab.dart';
import '../widgets/pointer_vis_tab.dart';
import '../widgets/progress_tab.dart';
import '../widgets/symbol_bar.dart';
import '../widgets/template_bar.dart';
import '../widgets/toolbar.dart';
import '../widgets/variables_tab.dart';
import '../widgets/watch_tab.dart';

class IdeScreen extends ConsumerStatefulWidget {
  const IdeScreen({super.key});

  @override
  ConsumerState<IdeScreen> createState() => _IdeScreenState();
}

class _IdeScreenState extends ConsumerState<IdeScreen> {
  final _editorKey = GlobalKey<EditorPanelState>();
  final _inputController = TextEditingController();

  @override
  void dispose() {
    _inputController.dispose();
    super.dispose();
  }

  void _insertText(String text) => _editorKey.currentState?.insertText(text);
  void _insertPair(String open, String close) => _editorKey.currentState?.insertPair(open, close);
  void _undo() => _editorKey.currentState?.undo();
  void _redo() => _editorKey.currentState?.redo();
  void _moveCursor(int offset) => _editorKey.currentState?.moveCursor(offset);
  void _scrollToLine(int line) => _editorKey.currentState?.scrollToLine(line);

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(ideProvider);
    final notifier = ref.read(ideProvider.notifier);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;

    final scaffoldBg = isDark ? const Color(0xff121212) : const Color(0xfff5f5f5);

    return Scaffold(
      backgroundColor: scaffoldBg,
      body: SafeArea(
        child: Stack(
          children: [
            Column(
              children: [
                _buildToolbar(state, notifier, isDark),
                Expanded(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    child: EditorPanel(key: _editorKey),
                  ),
                ),
                _buildSymbolBar(),
                _buildTemplateBar(state, notifier),
                _buildBottomPanel(state, notifier, isDark),
              ],
            ),
            if (state.showIntro)
              IntroOverlay(
                isDark: isDark,
                onDone: notifier.hideIntro,
              ),
          ],
        ),
      ),
      floatingActionButton: _buildFloatingButton(state, notifier),
      bottomSheet: state.isFloatingOpen ? _buildFloatingDrawer(state, notifier, isDark) : null,
    );
  }

  // ========== 工具栏 ==========

  Widget _buildToolbar(IdeState state, IdeNotifier notifier, bool isDark) {
    return Toolbar(
      state: state,
      notifier: notifier,
      isDark: isDark,
      onToggleTheme: () => ref.read(themeProvider.notifier).toggle(),
    );
  }

  // ========== 符号快捷栏 ==========

  Widget _buildSymbolBar() {
    return SymbolBar(
      onInsertPair: _insertPair,
      onInsertText: _insertText,
      onMoveCursor: _moveCursor,
      onUndo: _undo,
      onRedo: _redo,
    );
  }

  // ========== 模板快捷栏 ==========

  Widget _buildTemplateBar(IdeState state, IdeNotifier notifier) {
    return TemplateBar(
      templates: CodeTemplate.defaults,
      onSelectTemplate: _insertText,
    );
  }

  // ========== 底部面板 ==========

  Widget _buildBottomPanel(IdeState state, IdeNotifier notifier, bool isDark) {
    final panelBg = isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff);

    return HeightResizablePanel(
      height: state.bottomHeight,
      onHeightChanged: notifier.setBottomHeight,
      child: Container(
        decoration: BoxDecoration(
          color: panelBg,
          border: Border(
            top: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
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
                    final panelId = state.bottomSlots[index];
                    final item = PanelItem.fromId(panelId);
                    if (item == null) return const SizedBox.shrink();
                    final isActive = state.bottomActiveIndex == index;
                    return Expanded(
                      child: DraggablePanelTab(
                        item: item,
                        isActive: isActive,
                        badge: _getBadgeForPanel(panelId, state),
                        onTap: () => notifier.selectBottomTab(index),
                        onDoubleTap: () => notifier.removeBottomPanel(index),
                        data: PanelDragData(panelId: panelId, fromLocation: PanelLocation.bottom, fromIndex: index),
                        onAccept: (dragData) {
                          if (dragData.fromLocation == PanelLocation.bottom) {
                            notifier.swapBottomPanels(index, dragData.fromIndex);
                          } else {
                            notifier.moveToBottom(dragData.panelId);
                          }
                        },
                      ),
                    );
                  }),
                  // 底部空位 DropTarget（拖拽到底部区域上方添加）
                  Expanded(
                    flex: 1,
                    child: DragTarget<PanelDragData>(
                      onAcceptWithDetails: (details) {
                        notifier.moveToBottom(details.data.panelId);
                      },
                      builder: (context, candidateData, rejectedData) {
                        final isHovering = candidateData.isNotEmpty;
                        return Container(
                          margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
                          decoration: BoxDecoration(
                            color: isHovering ? Colors.blueAccent.withValues(alpha: 0.2) : null,
                            borderRadius: BorderRadius.circular(4),
                            border: isHovering ? Border.all(color: Colors.blueAccent) : null,
                          ),
                          child: const Center(
                            child: Icon(Icons.add, size: 16, color: Colors.grey),
                          ),
                        );
                      },
                    ),
                  ),
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
                child: _buildBottomTabContent(state, notifier, isDark),
              ),
            ),
          ],
        ),
      ),
    );
  }

  String? _getBadgeForPanel(String panelId, IdeState state) {
    switch (panelId) {
      case 'diagnostics':
        return state.diagnostics.isNotEmpty ? '${state.diagnostics.length}' : null;
      case 'algorithm':
        return state.algorithmMatches.isNotEmpty ? '${state.algorithmMatches.length}' : null;
      default:
        return null;
    }
  }

  Widget _buildBottomTabContent(IdeState state, IdeNotifier notifier, bool isDark) {
    if (state.bottomSlots.isEmpty) return const SizedBox.shrink();
    final panelId = state.bottomSlots[state.bottomActiveIndex.clamp(0, state.bottomSlots.length - 1)];
    switch (panelId) {
      case 'output':
        return _buildOutputTab(state, notifier, isDark);
      case 'diagnostics':
        return _buildDiagnosticsTab(state, notifier, isDark);
      case 'algorithm':
        return _buildAlgorithmTab(state, isDark);
      case 'knowledge':
        return _buildKnowledgeCardTab(state, isDark);
      case 'pointer':
        return _buildPointerVisTab(state, isDark);
      case 'arrayVis':
        return _buildArrayVisTab(state, isDark);
      case 'memory':
        return _buildMemoryTab(state, isDark);
      case 'variables':
        return _buildVariablesTab(state, isDark);
      case 'watch':
        return _buildWatchTab(state, isDark);
      case 'callstack':
        return _buildCallstackTab(state, isDark);
      case 'progress':
        return _buildProgressTab(state, isDark);
      default:
        return const SizedBox.shrink();
    }
  }

  // ========== 悬浮球 ==========

  Widget _buildFloatingButton(IdeState state, IdeNotifier notifier) {
    return FloatingActionButton(
      mini: true,
      backgroundColor: state.isFloatingOpen ? Colors.redAccent : Colors.blueAccent,
      onPressed: notifier.toggleFloating,
      child: Icon(state.isFloatingOpen ? Icons.close : Icons.bug_report, size: 20),
    );
  }

  Widget _buildFloatingDrawer(IdeState state, IdeNotifier notifier, bool isDark) {
    final panelBg = isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff);

    return Container(
      height: 320,
      decoration: BoxDecoration(
        color: panelBg,
        borderRadius: const BorderRadius.vertical(top: Radius.circular(12)),
        boxShadow: [BoxShadow(color: Colors.black.withValues(alpha: 0.2), blurRadius: 8)],
      ),
      child: Column(
        children: [
          // 拖拽手柄 + 关闭
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Row(
              children: [
                Container(width: 40, height: 4, decoration: BoxDecoration(color: Colors.grey, borderRadius: BorderRadius.circular(2))),
                const Spacer(),
                Text('调试面板', style: TextStyle(fontSize: 12, color: Colors.grey[600])),
                const Spacer(),
                InkWell(
                  onTap: notifier.closeFloating,
                  child: const Icon(Icons.close, size: 18, color: Colors.grey),
                ),
              ],
            ),
          ),
          // Tab 栏
          Container(
            height: 40,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: Row(
              children: [
                ...List.generate(state.floatingSlots.length, (index) {
                  final panelId = state.floatingSlots[index];
                  final item = PanelItem.fromId(panelId);
                  if (item == null) return const SizedBox.shrink();
                  final isActive = state.floatingActiveIndex == index;
                  return Expanded(
                    child: DraggablePanelTab(
                      item: item,
                      isActive: isActive,
                      onTap: () => notifier.selectFloatingTab(index),
                      onDoubleTap: () => notifier.removeFloatingPanel(index),
                      data: PanelDragData(panelId: panelId, fromLocation: PanelLocation.floating, fromIndex: index),
                      onAccept: (dragData) {
                        if (dragData.fromLocation == PanelLocation.floating) {
                          notifier.swapFloatingPanels(index, dragData.fromIndex);
                        } else {
                          notifier.moveToFloating(dragData.panelId);
                        }
                      },
                    ),
                  );
                }),
                // 悬浮球空位 DropTarget
                if (state.floatingSlots.length < 7)
                  Expanded(
                    child: DragTarget<PanelDragData>(
                      onAcceptWithDetails: (details) {
                        notifier.moveToFloating(details.data.panelId);
                      },
                      builder: (context, candidateData, rejectedData) {
                        final isHovering = candidateData.isNotEmpty;
                        return Container(
                          margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
                          decoration: BoxDecoration(
                            color: isHovering ? Colors.blueAccent.withValues(alpha: 0.2) : null,
                            borderRadius: BorderRadius.circular(4),
                            border: isHovering ? Border.all(color: Colors.blueAccent) : null,
                          ),
                          child: const Center(
                            child: Icon(Icons.add, size: 16, color: Colors.grey),
                          ),
                        );
                      },
                    ),
                  ),
              ],
            ),
          ),
          // 内容区域（支持水平滑动切换标签）
          Expanded(
            child: GestureDetector(
              onHorizontalDragEnd: (details) {
                const threshold = 300.0;
                final dx = details.velocity.pixelsPerSecond.dx;
                if (dx > threshold && state.floatingActiveIndex > 0) {
                  notifier.selectFloatingTab(state.floatingActiveIndex - 1);
                } else if (dx < -threshold &&
                    state.floatingActiveIndex < state.floatingSlots.length - 1) {
                  notifier.selectFloatingTab(state.floatingActiveIndex + 1);
                }
              },
              child: _buildFloatingTabContent(state, notifier, isDark),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFloatingTabContent(IdeState state, IdeNotifier notifier, bool isDark) {
    if (state.floatingSlots.isEmpty) return const SizedBox.shrink();
    final panelId = state.floatingSlots[state.floatingActiveIndex.clamp(0, state.floatingSlots.length - 1)];
    switch (panelId) {
      case 'output':
        return _buildOutputTab(state, notifier, isDark);
      case 'diagnostics':
        return _buildDiagnosticsTab(state, notifier, isDark);
      case 'algorithm':
        return _buildAlgorithmTab(state, isDark);
      case 'knowledge':
        return _buildKnowledgeCardTab(state, isDark);
      case 'pointer':
        return _buildPointerVisTab(state, isDark);
      case 'arrayVis':
        return _buildArrayVisTab(state, isDark);
      case 'memory':
        return _buildMemoryTab(state, isDark);
      case 'variables':
        return _buildVariablesTab(state, isDark);
      case 'watch':
        return _buildWatchTab(state, isDark);
      case 'callstack':
        return _buildCallstackTab(state, isDark);
      case 'progress':
        return _buildProgressTab(state, isDark);
      default:
        return const SizedBox.shrink();
    }
  }

  // ========== 各 Tab 内容 ==========

  Widget _buildOutputTab(IdeState state, IdeNotifier notifier, bool isDark) {
    return OutputTab(
      state: state,
      notifier: notifier,
      isDark: isDark,
      inputController: _inputController,
    );
  }

  Widget _buildDiagnosticsTab(IdeState state, IdeNotifier notifier, bool isDark) {
    return DiagnosticsTab(
      state: state,
      notifier: notifier,
      isDark: isDark,
      onScrollToLine: _scrollToLine,
      onUpdateSource: (src) => _editorKey.currentState?.setText(src),
    );
  }

  Widget _buildAlgorithmTab(IdeState state, bool isDark) {
    return AlgorithmTab(matches: state.algorithmMatches, isDark: isDark);
  }

  Widget _buildKnowledgeCardTab(IdeState state, bool isDark) {
    return KnowledgeCardTab(cards: state.knowledgeCards, isDark: isDark);
  }

  Widget _buildPointerVisTab(IdeState state, bool isDark) {
    return PointerVisTab(isDark: isDark);
  }

  Widget _buildArrayVisTab(IdeState state, bool isDark) {
    return ArrayVisTab(isDark: isDark);
  }

  Widget _buildWatchTab(IdeState state, bool isDark) {
    return WatchTab(watchExpressions: state.watchExpressions, isDark: isDark);
  }

  Widget _buildMemoryTab(IdeState state, bool isDark) {
    return MemoryTab(isDark: isDark);
  }

  Widget _buildVariablesTab(IdeState state, bool isDark) {
    return VariablesTab(isDark: isDark);
  }

  Widget _buildCallstackTab(IdeState state, bool isDark) {
    return const CallstackTab();
  }

  Widget _buildProgressTab(IdeState state, bool isDark) {
    return ProgressTab(state: state);
  }
}

// ========== 可拖拽高度的面板包装器 ==========

// ========== 知识卡片组件 ==========

// ========== 小型组件 ==========


}


