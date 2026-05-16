import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/code_template.dart';
import '../models/ide_state.dart';
import '../models/panel_item.dart';
import '../providers/ide_provider.dart';
import '../providers/theme_provider.dart';
import '../widgets/draggable_panel_tab.dart';
import '../widgets/editor_panel.dart';
import '../widgets/floating_orb_widget.dart';
import '../widgets/floating_panel_popup.dart';
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
import '../widgets/custom_keyboard.dart';
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
  bool _showKeyboard = false;
  bool _isSystemKeyboardActive = false;
  OverlayEntry? _orbOverlayEntry;
  OverlayEntry? _panelOverlayEntry;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _insertOrbOverlay();
    });
  }

  void _insertOrbOverlay() {
    if (!mounted) return;
    final overlay = Overlay.of(context);
    _orbOverlayEntry = OverlayEntry(
      builder: (context) => _buildOrbOverlay(),
    );
    overlay.insert(_orbOverlayEntry!);
  }

  void _insertPanelOverlay(String panelId) {
    _removePanelOverlay();
    if (!mounted) return;
    final overlay = Overlay.of(context);
    _panelOverlayEntry = OverlayEntry(
      builder: (context) => _buildPanelOverlay(panelId),
    );
    overlay.insert(_panelOverlayEntry!);
  }

  void _removePanelOverlay() {
    _panelOverlayEntry?.remove();
    _panelOverlayEntry = null;
  }

  @override
  void dispose() {
    _orbOverlayEntry?.remove();
    _panelOverlayEntry?.remove();
    _inputController.dispose();
    super.dispose();
  }

  /// 显示自定义键盘
  void _openKeyboard() {
    if (!_showKeyboard) {
      setState(() => _showKeyboard = true);
    }
  }

  /// 隐藏自定义键盘
  void _closeKeyboard() {
    if (_showKeyboard) {
      setState(() => _showKeyboard = false);
    }
  }

  /// 切换到系统键盘（用于中文输入）
  void _showSystemKeyboard() {
    setState(() => _isSystemKeyboardActive = true);
    _editorKey.currentState?.setReadOnly(false);
    SystemChannels.textInput.invokeMethod('TextInput.show');
  }

  /// 切换回自定义键盘（英文/代码输入模式）
  void _showCustomKeyboard() {
    SystemChannels.textInput.invokeMethod('TextInput.hide');
    setState(() => _isSystemKeyboardActive = false);
    _editorKey.currentState?.setReadOnly(true);
  }

  void _insertText(String text) => _editorKey.currentState?.insertText(text);
  void _insertPair(String open, String close) => _editorKey.currentState?.insertPair(open, close);
  void _undo() => _editorKey.currentState?.undo();
  void _redo() => _editorKey.currentState?.redo();
  void _moveCursor(int offset) => _editorKey.currentState?.moveCursor(offset);
  void _scrollToLine(int line) => _editorKey.currentState?.scrollToLine(line);
  void _backspace() => _editorKey.currentState?.backspace();
  void _insertNewline() => _editorKey.currentState?.insertNewline();

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(ideProvider);
    final notifier = ref.read(ideProvider.notifier);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;

    final scaffoldBg = isDark ? const Color(0xff121212) : const Color(0xfff5f5f5);
    final showCustomKeyboard = _showKeyboard && !_isSystemKeyboardActive;

    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: scaffoldBg,
      body: Stack(
        children: [
          SafeArea(
            child: Stack(
              children: [
                Column(
                  children: [
                    _buildToolbar(state, notifier, isDark),
                    Expanded(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 8),
                        child: EditorPanel(
                          key: _editorKey,
                          onTap: _openKeyboard,
                        ),
                      ),
                    ),
                    _buildTemplateBar(state, notifier),
                    _buildBottomPanel(state, notifier, isDark),
                  ],
                ),
                // 自定义键盘：编辑器聚焦且未切换系统键盘时显示
                if (showCustomKeyboard)
                  Positioned(
                    left: 0,
                    right: 0,
                    bottom: 0,
                    child: FocusScope(
                      canRequestFocus: false,
                      child: CustomKeyboard(
                        onInsertText: _insertText,
                        onInsertPair: _insertPair,
                        onMoveCursor: _moveCursor,
                        onBackspace: _backspace,
                        onEnter: _insertNewline,
                        onTab: () => _insertText('    '),
                        onUndo: _undo,
                        onRedo: _redo,
                        onDone: _closeKeyboard,
                        onToggleSystemKeyboard: _showSystemKeyboard,
                        isSystemKeyboardActive: false,
                      ),
                    ),
                  ),
                // 系统键盘激活时，提供一个悬浮按钮可切回自定义键盘
                if (_isSystemKeyboardActive)
                  Positioned(
                    right: 16,
                    bottom: MediaQuery.of(context).viewInsets.bottom + 16,
                    child: FloatingActionButton(
                      mini: true,
                      backgroundColor: Colors.blueAccent,
                      onPressed: _showCustomKeyboard,
                      child: const Text('英', style: TextStyle(fontSize: 14, color: Colors.white)),
                    ),
                  ),
                if (state.showIntro)
                  IntroOverlay(
                    isDark: isDark,
                    onDone: notifier.hideIntro,
                  ),
              ],
            ),
          ),
          // 悬浮球通过 OverlayEntry 渲染，不放在 body Stack 中
        ],
      ),
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

  // ========== Overlay 悬浮球与弹窗 ==========

  Widget _buildOrbOverlay() {
    return Consumer(
      builder: (context, ref, child) {
        final state = ref.watch(ideProvider);
        final notifier = ref.read(ideProvider.notifier);
        return FloatingOrbWidget(
          isMenuOpen: state.isFloatingOpen,
          menuItems: state.floatingSlots,
          onToggleMenu: notifier.toggleFloating,
          onSelectPanel: (panelId) {
            notifier.openFloatingPanel(panelId);
            _insertPanelOverlay(panelId);
          },
          onCloseMenu: notifier.closeFloating,
        );
      },
    );
  }

  Widget _buildPanelOverlay(String panelId) {
    return Consumer(
      builder: (context, ref, child) {
        final state = ref.watch(ideProvider);
        final notifier = ref.read(ideProvider.notifier);
        final isDark = ref.watch(themeProvider) == ThemeMode.dark;
        return FloatingPanelPopup(
          panelId: panelId,
          isDark: isDark,
          onClose: () {
            notifier.closeFloatingPanel();
            _removePanelOverlay();
          },
          child: _buildPanelContentById(panelId, state, notifier, isDark),
        );
      },
    );
  }

  Widget _buildPanelContentById(String panelId, IdeState state, IdeNotifier notifier, bool isDark) {
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
