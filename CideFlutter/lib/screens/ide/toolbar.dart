import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/ide_provider.dart';
import '../../providers/theme_provider.dart';
import '../../widgets/toolbar.dart';

/// IDE 顶部工具栏包装组件。
///
/// 负责订阅 [ideProvider] 与 [themeProvider]，并在键盘弹出时通过
/// [animation] 平滑收起。
class IdeToolbar extends ConsumerWidget {
  final Animation<double> animation;

  const IdeToolbar({super.key, required this.animation});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(ideProvider);
    final notifier = ref.read(ideProvider.notifier);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;

    return SizeTransition(
      sizeFactor: animation,
      axisAlignment: -1,
      child: Toolbar(
        state: state,
        notifier: notifier,
        isDark: isDark,
        onToggleTheme: () => ref.read(themeProvider.notifier).toggle(),
      ),
    );
  }
}
