import 'dart:io' show Platform;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/code_template.dart';
import '../models/panel_item.dart';
import '../providers/ide_provider.dart';
import '../providers/unified_provider.dart';
import '../providers/theme_provider.dart';
import '../widgets/draggable_panel_tab.dart';
import '../widgets/editor_panel_v2.dart';
import '../widgets/floating_orb_widget.dart';
import '../widgets/floating_panel_popup.dart';
import '../widgets/height_resizable_panel.dart';
import '../widgets/file_tab_bar.dart';
import '../widgets/intro_overlay.dart';
import '../widgets/linked_list_vis_tab.dart';
import '../widgets/tree_vis_tab.dart';
import '../widgets/panel_drag_data.dart';
import '../widgets/algorithm_tab.dart';
import '../widgets/array_vis_tab.dart';
import '../widgets/callstack_tab.dart';
import '../widgets/diagnostics_tab.dart';
import '../widgets/intent_inference_panel.dart';
import '../widgets/knowledge_card_tab.dart';
import '../widgets/memory_tab.dart';
import '../widgets/output_tab.dart';
import '../widgets/pointer_vis_tab.dart';
import '../widgets/progress_tab.dart';
import '../widgets/custom_keyboard.dart';
import '../widgets/template_bar.dart';
import '../widgets/template_param_dialog.dart';
import '../widgets/template_tutorial_panel.dart';
import '../widgets/execution_control_panel.dart';
import '../widgets/toolbar.dart';
import '../widgets/breakpoints_tab.dart';
import '../widgets/var_history_tab.dart';
import '../widgets/variables_tab.dart';
import '../widgets/watch_tab.dart';

class IdeScreen extends ConsumerStatefulWidget {
  const IdeScreen({super.key});

  @override
  ConsumerState<IdeScreen> createState() => _IdeScreenState();
}

class _RunIntent extends Intent {
  const _RunIntent();
}

class _StepIntent extends Intent {
  const _StepIntent();
}

class _ToggleBreakpointIntent extends Intent {
  const _ToggleBreakpointIntent();
}

class _StopIntent extends Intent {
  const _StopIntent();
}

class _IdeScreenState extends ConsumerState<IdeScreen>
    with SingleTickerProviderStateMixin {
  final _editorKey = GlobalKey<EditorPanelV2State>();

  dynamic get _editor => _editorKey.currentState;

  final _inputController = TextEditingController();
  bool _showKeyboard = false;
  bool _isSystemKeyboardActive = false;
  OverlayEntry? _orbOverlayEntry;
  OverlayEntry? _panelOverlayEntry;
  late final AnimationController _barsAnimationController;
  late final Animation<double> _barsAnimation;

  // ========== 快捷键 Intent ==========
  void _handleRun() {
    final unified = ref.read(unifiedProvider);
    final unifiedNotifier = ref.read(unifiedProvider.notifier);
    if (unified.phase == ExecutionPhase.idle) {
      ref.read(ideProvider.notifier).compile();
    } else if (unified.canPlay && !unified.isPlaying) {
      unifiedNotifier.resume();
    }
  }

  void _handleStep() {
    final unified = ref.read(unifiedProvider);
    if (unified.canStep) {
      ref.read(unifiedProvider.notifier).stepNext();
    }
  }

  void _handleToggleBreakpoint() {
    final line = _editor?.getCurrentLine() ?? 0;
    if (line > 0) {
      ref.read(ideProvider.notifier).toggleBreakpoint(line);
    }
  }

  void _handleStop() {
    ref.read(unifiedProvider.notifier).onCodeChanged();
  }

  @override
  void initState() {
    super.initState();
    _barsAnimationController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 250),
      value: 1.0,
    );
    _barsAnimation = CurvedAnimation(
      parent: _barsAnimationController,
      curve: Curves.easeInOut,
    );
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _insertOrbOverlay();
    });
  }

  void _insertOrbOverlay() {
    if (!mounted) return;
    final overlay = Overlay.of(context);
    _orbOverlayEntry = OverlayEntry(builder: (context) => _buildOrbOverlay());
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
    _barsAnimationController.dispose();
    _orbOverlayEntry?.remove();
    _panelOverlayEntry?.remove();
    _inputController.dispose();
    super.dispose();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    // 不再根据 viewInsets 自动切回自定义键盘。
    // 键盘切换只由用户明确操作（点击「完成」/「英」按钮）触发，
    // 避免 IME 候选词面板弹出/收起时误判为键盘隐藏。
  }

  void _syncBarsAnimation() {
    final viewInsetsBottom = MediaQuery.of(context).viewInsets.bottom;
    final isSystemKeyboardVisible = viewInsetsBottom > 50;
    final isCustomKeyboardVisible = _showKeyboard && !_isSystemKeyboardActive;
    final isAnyKeyboardVisible =
        isCustomKeyboardVisible || isSystemKeyboardVisible;
    final target = isAnyKeyboardVisible ? 0.0 : 1.0;
    if ((_barsAnimationController.value - target).abs() > 0.01) {
      _barsAnimationController.animateTo(target);
    }
  }

  /// 打开键盘（根据当前模式显示对应键盘）
  void _openKeyboard() {
    debugPrint('[IdeScreen] _openKeyboard: system=$_isSystemKeyboardActive');
    if (_showKeyboard) return;
    setState(() => _showKeyboard = true);
    if (_isSystemKeyboardActive) {
      _editor?.showSystemKeyboard();
      _editor?.setReadOnly(false);
    }
  }

  /// 隐藏键盘（不改变键盘模式）
  void _closeKeyboard() {
    debugPrint('[IdeScreen] _closeKeyboard');
    if (!_showKeyboard) return;
    setState(() => _showKeyboard = false);
    if (_isSystemKeyboardActive) {
      SystemChannels.textInput.invokeMethod('TextInput.hide');
    }
  }

  /// 关闭所有键盘（隐藏即可，绝不抢夺模式）
  void _closeAllKeyboards() {
    debugPrint('[IdeScreen] _closeAllKeyboards');
    _closeKeyboard();
  }

  /// 切换到系统键盘（用于中文输入）——唯一入口
  void _showSystemKeyboard() {
    debugPrint('[IdeScreen] _showSystemKeyboard');
    setState(() => _isSystemKeyboardActive = true);
    // 先解除 readOnly，确保 _attachInputConnection 的 _readOnly 拦截不会生效
    _editor?.setReadOnly(false);
    _editor?.showSystemKeyboard();
  }

  /// 切换回自定义键盘（英文/代码输入模式）——唯一入口
  void _showCustomKeyboard() {
    debugPrint('[IdeScreen] _showCustomKeyboard');
    SystemChannels.textInput.invokeMethod('TextInput.hide');
    setState(() => _isSystemKeyboardActive = false);
    // 先断开 input connection，再设 readOnly，避免 EditableText 在 readOnly
    // 切换过程中残留 system IME 焦点。
    _editor?.showCustomKeyboard();
    _editor?.setReadOnly(true);
  }

  void _showAddFileDialog(BuildContext context, IdeNotifier notifier) {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder:
          (context) => AlertDialog(
            title: const Text('新建文件'),
            content: TextField(
              controller: controller,
              autofocus: true,
              decoration: const InputDecoration(
                hintText: '输入文件名（如 utils.c）',
                border: OutlineInputBorder(),
              ),
              onSubmitted: (value) {
                if (value.isNotEmpty) {
                  notifier.addFile(value);
                }
                Navigator.of(context).pop();
              },
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('取消'),
              ),
              TextButton(
                onPressed: () {
                  final value = controller.text.trim();
                  if (value.isNotEmpty) {
                    notifier.addFile(value);
                  }
                  Navigator.of(context).pop();
                },
                child: const Text('确定'),
              ),
            ],
          ),
    );
  }

  void _insertText(String text) => _editor?.insertText(text);
  void _insertPair(String open, String close) =>
      _editor?.insertPair(open, close);
  void _undo() => _editor?.undo();
  void _redo() => _editor?.redo();
  void _moveCursor(int offset) => _editor?.moveCursor(offset);
  void _scrollToLine(int line) => _editor?.scrollToLine(line);
  void _backspace() => _editor?.backspace();
  void _insertNewline() => _editor?.insertNewline();

  void _handleTemplateSelect(CodeTemplate template) {
    final notifier = ref.read(ideProvider.notifier);

    // 无参数且无教程：直接插入（旧行为）
    if (template.params.isEmpty && template.tutorialSteps.isEmpty) {
      _insertText(template.code);
      return;
    }

    // 有参数：先弹参数对话框
    if (template.params.isNotEmpty) {
      showTemplateParamDialog(
        context: context,
        template: template,
        onConfirm: (params) {
          final generated = template.buildCode(params);
          if (template.tutorialSteps.isNotEmpty) {
            // 启动教程
            notifier.startTutorial(template, generated);
            _scrollToTutorialFocus();
          } else {
            // 无教程，直接插入
            _insertText(generated);
          }
        },
      );
      return;
    }

    // 无参数但有教程
    notifier.startTutorial(template, template.code);
    _scrollToTutorialFocus();
  }

  void _scrollToTutorialFocus() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final tutorial = ref.read(ideProvider).activeTutorial;
      if (tutorial != null && tutorial.focusLines.isNotEmpty) {
        _scrollToLine(tutorial.focusLines.first);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(ideProvider);
    final notifier = ref.read(ideProvider.notifier);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;

    final scaffoldBg =
        isDark ? const Color(0xff121212) : const Color(0xfff5f5f5);
    final showCustomKeyboard = _showKeyboard && !_isSystemKeyboardActive;

    // 监听错误信息并弹出提示
    ref.listen(ideProvider, (prev, next) {
      if (next.error != null && next.error != prev?.error) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text(next.error!),
                duration: const Duration(seconds: 3),
                behavior: SnackBarBehavior.floating,
              ),
            );
          }
        });
      }
    });

    // 检测系统键盘真实可见性（仅读取，副作用已移至 didChangeDependencies）
    final viewInsetsBottom = MediaQuery.of(context).viewInsets.bottom;
    final isSystemKeyboardReallyVisible = viewInsetsBottom > 50;
    // 桌面端物理键盘不会更新 viewInsets，只要处于系统键盘模式就显示切换按钮
    final isDesktop =
        Platform.isWindows || Platform.isMacOS || Platform.isLinux;
    final showSystemKeyboardToggle =
        _isSystemKeyboardActive && (isDesktop || isSystemKeyboardReallyVisible);

    // 同步上下栏动画
    _syncBarsAnimation();

    final shortcuts = <ShortcutActivator, Intent>{
      const SingleActivator(LogicalKeyboardKey.f5): const _RunIntent(),
      const SingleActivator(LogicalKeyboardKey.f10): const _StepIntent(),
      const SingleActivator(LogicalKeyboardKey.f9):
          const _ToggleBreakpointIntent(),
      const SingleActivator(LogicalKeyboardKey.f5, shift: true):
          const _StopIntent(),
    };

    final actions = <Type, Action<Intent>>{
      _RunIntent: CallbackAction<_RunIntent>(onInvoke: (_) => _handleRun()),
      _StepIntent: CallbackAction<_StepIntent>(onInvoke: (_) => _handleStep()),
      _ToggleBreakpointIntent: CallbackAction<_ToggleBreakpointIntent>(
        onInvoke: (_) => _handleToggleBreakpoint(),
      ),
      _StopIntent: CallbackAction<_StopIntent>(onInvoke: (_) => _handleStop()),
    };

    return Shortcuts(
      shortcuts: shortcuts,
      child: Actions(
        actions: actions,
        child: Focus(
          autofocus: true,
          child: Scaffold(
            resizeToAvoidBottomInset: false,
            backgroundColor: scaffoldBg,
            body: Stack(
              children: [
                SafeArea(
                  child: Stack(
                    children: [
                      Column(
                        children: [
                          // 顶部工具栏：键盘弹出时平滑收起
                          SizeTransition(
                            sizeFactor: _barsAnimation,
                            axisAlignment: -1,
                            child: _buildToolbar(state, notifier, isDark),
                          ),
                          _buildExecutionControl(state, notifier),
                          FileTabBar(
                            files: state.files,
                            currentFile: state.currentFile,
                            onSwitch:
                                (filename) => notifier.switchFile(filename),
                            onClose:
                                (filename) => notifier.removeFile(filename),
                            onAdd: () => _showAddFileDialog(context, notifier),
                          ),
                          Expanded(
                            child: Padding(
                              padding: const EdgeInsets.symmetric(
                                horizontal: 8,
                              ),
                              child: EditorPanelV2(
                                key: _editorKey,
                                onTap: _openKeyboard,
                                onBlankTap: _closeAllKeyboards,
                                onDismissKeyboard: _closeAllKeyboards,
                              ),
                            ),
                          ),
                          // 教程激活时显示教程面板，否则显示模板栏+底部面板
                          if (state.activeTutorial != null)
                            TemplateTutorialPanel(
                              templateName: state.activeTutorial!.templateKey,
                              currentStep: state.activeTutorial!.stepIndex,
                              totalSteps: state.activeTutorial!.steps.length,
                              step:
                                  state.activeTutorial!.steps[state
                                      .activeTutorial!
                                      .stepIndex],
                              isDark: isDark,
                              onNext: () {
                                notifier.nextTutorialStep();
                                _scrollToTutorialFocus();
                              },
                              onPrev: () {
                                notifier.prevTutorialStep();
                                _scrollToTutorialFocus();
                              },
                              onSkip: notifier.skipTutorial,
                              onRun: () => notifier.completeTutorial(),
                            )
                          else ...[
                            // 模板栏：键盘弹出时平滑收起
                            SizeTransition(
                              sizeFactor: _barsAnimation,
                              axisAlignment: 1,
                              child: _buildTemplateBar(state, notifier),
                            ),
                            // 底部面板：键盘弹出时平滑收起
                            SizeTransition(
                              sizeFactor: _barsAnimation,
                              axisAlignment: 1,
                              child: _buildBottomPanel(state, notifier, isDark),
                            ),
                          ],
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
                      // 系统键盘激活时，提供悬浮按钮切回自定义键盘
                      if (showSystemKeyboardToggle)
                        Positioned(
                          right: 16,
                          bottom: viewInsetsBottom + 16,
                          child: FloatingActionButton(
                            mini: true,
                            backgroundColor: Colors.blueAccent,
                            onPressed: _showCustomKeyboard,
                            child: const Text(
                              '英',
                              style: TextStyle(
                                fontSize: 14,
                                color: Colors.white,
                              ),
                            ),
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
          ),
        ),
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

  // ========== 执行控制面板 ==========

  Widget _buildExecutionControl(IdeState state, IdeNotifier notifier) {
    return ExecutionControlPanel(
      onRun: () => notifier.compile(),
      onScrollToLine: _scrollToLine,
    );
  }

  // ========== 模板快捷栏 ==========

  Widget _buildTemplateBar(IdeState state, IdeNotifier notifier) {
    return TemplateBar(
      templates: CodeTemplate.defaults,
      onSelectTemplate: _handleTemplateSelect,
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

                        data: PanelDragData(
                          panelId: panelId,
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

  Widget _buildBottomTabContent(
    IdeState state,
    IdeNotifier notifier,
    bool isDark,
  ) {
    if (state.bottomSlots.isEmpty) return const SizedBox.shrink();
    final panelId =
        state.bottomSlots[state.bottomActiveIndex.clamp(
          0,
          state.bottomSlots.length - 1,
        )];
    switch (panelId) {
      case 'output':
        return _buildOutputTab(state, notifier, isDark);
      case 'diagnostics':
        return _buildDiagnosticsTab(state, notifier, isDark);
      case 'algorithm':
        return _buildAlgorithmTab(state, isDark);
      case 'intent':
        return _buildIntentTab(state);
      case 'knowledge':
        return _buildKnowledgeCardTab(state, isDark);
      case 'pointer':
        return _buildPointerVisTab(state, isDark);
      case 'arrayVis':
        return _buildArrayVisTab(state, isDark);
      case 'linkedListVis':
        return _buildLinkedListVisTab(isDark);
      case 'treeVis':
        return _buildTreeVisTab(isDark);
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
      case 'varHistory':
        return _buildVarHistoryTab(isDark);
      case 'breakpoints':
        return _buildBreakpointsTab(state, isDark);
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
          onDragAccept: (dragData) {
            if (dragData.fromLocation == PanelLocation.bottom) {
              // 拖到边缘/padding 区域，未落在具体菜单项上
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('未识别到可交换的目标位置'),
                  duration: Duration(seconds: 1),
                ),
              );
            }
          },
          onSwapWithFloatingItem: (dragData, targetIndex) {
            if (dragData.fromLocation == PanelLocation.bottom) {
              // 底部 → 悬浮球具体项，交换
              notifier.swapBottomWithFloatingItem(
                dragData.panelId,
                targetIndex,
              );
            }
          },
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

  Widget _buildPanelContentById(
    String panelId,
    IdeState state,
    IdeNotifier notifier,
    bool isDark,
  ) {
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
      case 'linkedListVis':
        return _buildLinkedListVisTab(isDark);
      case 'treeVis':
        return _buildTreeVisTab(isDark);
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
      case 'varHistory':
        return _buildVarHistoryTab(isDark);
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

  Widget _buildDiagnosticsTab(
    IdeState state,
    IdeNotifier notifier,
    bool isDark,
  ) {
    return DiagnosticsTab(
      state: state,
      notifier: notifier,
      isDark: isDark,
      onScrollToLine: _scrollToLine,
      onUpdateSource: (src) => _editor?.setText(src),
    );
  }

  Widget _buildAlgorithmTab(IdeState state, bool isDark) {
    return AlgorithmTab(matches: state.algorithmMatches, isDark: isDark);
  }

  Widget _buildIntentTab(IdeState state) {
    return IntentInferencePanel(scores: state.intentScores);
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

  Widget _buildLinkedListVisTab(bool isDark) {
    return LinkedListVisTab(isDark: isDark);
  }

  Widget _buildTreeVisTab(bool isDark) {
    return TreeVisTab(isDark: isDark);
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
    return CallstackTab(isDark: isDark);
  }

  Widget _buildVarHistoryTab(bool isDark) {
    return VarHistoryTab(isDark: isDark);
  }

  Widget _buildProgressTab(IdeState state, bool isDark) {
    return ProgressTab(state: state);
  }

  Widget _buildBreakpointsTab(IdeState state, bool isDark) {
    return BreakpointsTab(
      state: state,
      isDark: isDark,
      onScrollToLine: _scrollToLine,
    );
  }
}

// ========== 可拖拽高度的面板包装器 ==========

// ========== 知识卡片组件 ==========

// ========== 小型组件 ==========
