import 'dart:io' show Platform;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/panel_item.dart';
import '../providers/ide_provider.dart';
import '../providers/unified_provider.dart';
import '../providers/theme_provider.dart';
import '../widgets/custom_keyboard.dart';
import '../widgets/editor_panel_v2.dart';
import '../widgets/execution_control_panel.dart';
import '../widgets/floating_orb_widget.dart';
import '../widgets/floating_panel_popup.dart';
import '../widgets/intro_overlay.dart';
import '../widgets/template_tutorial_panel.dart';
import 'ide/bottom_panel.dart';
import 'ide/editor_area.dart';
import 'ide/template_bar.dart';
import 'ide/toolbar.dart';

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
                          IdeToolbar(animation: _barsAnimation),
                          _buildExecutionControl(state, notifier),
                          IdeEditorArea(
                            editorKey: _editorKey,
                            onTap: _openKeyboard,
                            onBlankTap: _closeAllKeyboards,
                            onDismissKeyboard: _closeAllKeyboards,
                            onAddFile: () => _showAddFileDialog(context, notifier),
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
                            IdeTemplateBar(
                              animation: _barsAnimation,
                              onInsertText: _insertText,
                              onScrollToLine: _scrollToLine,
                            ),
                            // 底部面板：键盘弹出时平滑收起
                            IdeBottomPanel(
                              animation: _barsAnimation,
                              inputController: _inputController,
                              onScrollToLine: _scrollToLine,
                              onUpdateSource: (src) => _editor?.setText(src),
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

  // ========== 执行控制面板 ==========

  Widget _buildExecutionControl(IdeState state, IdeNotifier notifier) {
    return ExecutionControlPanel(
      onRun: () => notifier.compile(),
      onScrollToLine: _scrollToLine,
    );
  }

  // ========== 模板快捷栏 ==========

  // ========== 底部面板 ==========

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

          child: PanelContent(
            panelId: panelId,
            state: state,
            notifier: notifier,
            isDark: isDark,
            inputController: _inputController,
            onScrollToLine: _scrollToLine,
            onUpdateSource: (src) => _editor?.setText(src),
          ),
        );
      },
    );
  }

}
