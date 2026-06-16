import 'dart:io' show Platform;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/ide_provider.dart';
import '../../providers/unified_provider.dart';
import '../../widgets/custom_keyboard.dart';
import '../../widgets/editor_panel_v2.dart';

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

/// IDE 键盘与快捷键处理组件。
///
/// 负责：
/// - 包装 [Shortcuts] / [Actions] / [Focus]，提供 F5/F9/F10/Shift+F5 等快捷键。
/// - 根据 [showKeyboard] 与 [isSystemKeyboardActive] 渲染自定义键盘或系统键盘切回按钮。
/// - 将自定义键盘的输入操作转发给编辑器。
class KeyboardHandler extends ConsumerWidget {
  final GlobalKey<EditorPanelV2State> editorKey;
  final bool showKeyboard;
  final bool isSystemKeyboardActive;
  final VoidCallback onCloseKeyboard;
  final VoidCallback onShowSystemKeyboard;
  final VoidCallback onShowCustomKeyboard;
  final Widget child;

  const KeyboardHandler({
    super.key,
    required this.editorKey,
    required this.showKeyboard,
    required this.isSystemKeyboardActive,
    required this.onCloseKeyboard,
    required this.onShowSystemKeyboard,
    required this.onShowCustomKeyboard,
    required this.child,
  });

  dynamic get _editor => editorKey.currentState;

  void _handleRun(WidgetRef ref) {
    final unified = ref.read(unifiedProvider);
    final unifiedNotifier = ref.read(unifiedProvider.notifier);
    if (unified.phase == ExecutionPhase.idle) {
      ref.read(ideProvider.notifier).compile();
    } else if (unified.canPlay && !unified.isPlaying) {
      unifiedNotifier.resume();
    }
  }

  void _handleStep(WidgetRef ref) {
    final unified = ref.read(unifiedProvider);
    if (unified.canStep) {
      ref.read(unifiedProvider.notifier).stepNext();
    }
  }

  void _handleToggleBreakpoint(WidgetRef ref) {
    final line = _editor?.getCurrentLine() ?? 0;
    if (line > 0) {
      ref.read(ideProvider.notifier).toggleBreakpoint(line);
    }
  }

  void _handleStop(WidgetRef ref) {
    ref.read(unifiedProvider.notifier).onCodeChanged();
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final shortcuts = <ShortcutActivator, Intent>{
      const SingleActivator(LogicalKeyboardKey.f5): const _RunIntent(),
      const SingleActivator(LogicalKeyboardKey.f10): const _StepIntent(),
      const SingleActivator(LogicalKeyboardKey.f9):
          const _ToggleBreakpointIntent(),
      const SingleActivator(LogicalKeyboardKey.f5, shift: true):
          const _StopIntent(),
    };

    final actions = <Type, Action<Intent>>{
      _RunIntent: CallbackAction<_RunIntent>(
        onInvoke: (_) => _handleRun(ref),
      ),
      _StepIntent: CallbackAction<_StepIntent>(
        onInvoke: (_) => _handleStep(ref),
      ),
      _ToggleBreakpointIntent: CallbackAction<_ToggleBreakpointIntent>(
        onInvoke: (_) => _handleToggleBreakpoint(ref),
      ),
      _StopIntent: CallbackAction<_StopIntent>(
        onInvoke: (_) => _handleStop(ref),
      ),
    };

    final viewInsetsBottom = MediaQuery.of(context).viewInsets.bottom;
    final isSystemKeyboardReallyVisible = viewInsetsBottom > 50;
    final isDesktop =
        Platform.isWindows || Platform.isMacOS || Platform.isLinux;
    final showSystemKeyboardToggle =
        isSystemKeyboardActive && (isDesktop || isSystemKeyboardReallyVisible);
    final showCustomKeyboard = showKeyboard && !isSystemKeyboardActive;

    return Shortcuts(
      shortcuts: shortcuts,
      child: Actions(
        actions: actions,
        child: Focus(
          autofocus: true,
          child: Stack(
            children: [
              child,
              // 自定义键盘：编辑器聚焦且未切换系统键盘时显示
              if (showCustomKeyboard)
                Positioned(
                  left: 0,
                  right: 0,
                  bottom: 0,
                  child: FocusScope(
                    canRequestFocus: false,
                    child: CustomKeyboard(
                      onInsertText: (text) => _editor?.insertText(text),
                      onInsertPair: (open, close) =>
                          _editor?.insertPair(open, close),
                      onMoveCursor: (offset) => _editor?.moveCursor(offset),
                      onBackspace: () => _editor?.backspace(),
                      onEnter: () => _editor?.insertNewline(),
                      onTab: () => _editor?.insertText('    '),
                      onUndo: () => _editor?.undo(),
                      onRedo: () => _editor?.redo(),
                      onDone: onCloseKeyboard,
                      onToggleSystemKeyboard: onShowSystemKeyboard,
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
                    onPressed: onShowCustomKeyboard,
                    child: const Text(
                      '英',
                      style: TextStyle(
                        fontSize: 14,
                        color: Colors.white,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
