import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/ide_provider.dart';
import '../../widgets/editor_panel_v2.dart';
import '../../widgets/file_tab_bar.dart';

/// IDE 编辑器区域组件。
///
/// 组合 [FileTabBar] 与 [EditorPanelV2]，负责文件切换、关闭与新建文件入口，
/// 并将编辑器触控事件回调给外部处理。
class IdeEditorArea extends ConsumerWidget {
  final GlobalKey<EditorPanelV2State> editorKey;
  final VoidCallback onTap;
  final VoidCallback onBlankTap;
  final VoidCallback onDismissKeyboard;
  final VoidCallback onAddFile;

  const IdeEditorArea({
    super.key,
    required this.editorKey,
    required this.onTap,
    required this.onBlankTap,
    required this.onDismissKeyboard,
    required this.onAddFile,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(ideProvider);
    final notifier = ref.read(ideProvider.notifier);

    return Expanded(
      child: Column(
        children: [
          FileTabBar(
            files: state.files,
            currentFile: state.currentFile,
            onSwitch: (filename) => notifier.switchFile(filename),
            onClose: (filename) => notifier.removeFile(filename),
            onAdd: onAddFile,
          ),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: EditorPanelV2(
                key: editorKey,
                onTap: onTap,
                onBlankTap: onBlankTap,
                onDismissKeyboard: onDismissKeyboard,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
