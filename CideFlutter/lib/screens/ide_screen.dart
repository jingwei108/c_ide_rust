import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/ide_provider.dart';
import '../providers/theme_provider.dart';
import '../widgets/editor_panel_v2.dart';
import '../widgets/execution_control_panel.dart';
import '../widgets/intro_overlay.dart';
import '../widgets/template_tutorial_panel.dart';
import 'ide/bottom_panel.dart';
import 'ide/editor_area.dart';
import 'ide/floating_orb_area.dart';
import 'ide/keyboard_handler.dart';
import 'ide/template_bar.dart';
import 'ide/toolbar.dart';

class IdeScreen extends ConsumerStatefulWidget {
  const IdeScreen({super.key});

  @override
  ConsumerState<IdeScreen> createState() => _IdeScreenState();
}

class _IdeScreenState extends ConsumerState<IdeScreen>
    with SingleTickerProviderStateMixin {
  final _editorKey = GlobalKey<EditorPanelV2State>();

  dynamic get _editor => _editorKey.currentState;

  final _inputController = TextEditingController();
  bool _showKeyboard = false;
  bool _isSystemKeyboardActive = false;
  late final AnimationController _barsAnimationController;
  late final Animation<double> _barsAnimation;

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
  }

  @override
  void dispose() {
    _barsAnimationController.dispose();
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

  void _scrollToLine(int line) => _editor?.scrollToLine(line);

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

    // 同步上下栏动画
    _syncBarsAnimation();

    return Scaffold(
            resizeToAvoidBottomInset: false,
            backgroundColor: scaffoldBg,
            body: Stack(
              children: [
                SafeArea(
                  child: Stack(
                    children: [
                      KeyboardHandler(
                        editorKey: _editorKey,
                        showKeyboard: _showKeyboard,
                        isSystemKeyboardActive: _isSystemKeyboardActive,
                        onCloseKeyboard: _closeKeyboard,
                        onShowSystemKeyboard: _showSystemKeyboard,
                        onShowCustomKeyboard: _showCustomKeyboard,
                        child: Column(
                          children: [
                            // 顶部工具栏：键盘弹出时平滑收起
                            IdeToolbar(animation: _barsAnimation),
                          ExecutionControlPanel(
                            onRun: () => notifier.compile(),
                            onScrollToLine: _scrollToLine,
                          ),
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
                    ),
                    if (state.showIntro)
                        IntroOverlay(
                          isDark: isDark,
                          onDone: notifier.hideIntro,
                        ),
                    ],
                  ),
                ),
                FloatingOrbArea(
                  inputController: _inputController,
                  onScrollToLine: _scrollToLine,
                  onUpdateSource: (src) => _editor?.setText(src),
                ),
              ],
            ),
          );
  }

  // ========== 模板快捷栏 ==========

  // ========== 底部面板 ==========
}
