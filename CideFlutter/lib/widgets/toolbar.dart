import 'package:flutter/material.dart';
import '../models/ide_state.dart';
import '../providers/ide_notifier.dart';
import 'tool_button.dart';

class Toolbar extends StatelessWidget {
  final IdeState state;
  final IdeNotifier notifier;
  final bool isDark;
  final VoidCallback onToggleTheme;

  const Toolbar({
    super.key,
    required this.state,
    required this.notifier,
    required this.isDark,
    required this.onToggleTheme,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: BoxDecoration(
        color: isDark ? const Color(0xff1e1e1e) : const Color(0xfff5f5f5),
        border: Border(
          bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
        ),
      ),
      child: Row(
        children: [
          ToolButton(
            icon: Icons.play_arrow,
            color: Colors.green,
            onPressed: state.isRunning && !state.isStepMode ? null : notifier.run,
          ),
          if (state.isRunning)
            ToolButton(
              icon: Icons.stop,
              color: Colors.red,
              onPressed: notifier.reset,
            ),
          ToolButton(
            icon: Icons.skip_next,
            color: Colors.orange,
            onPressed: notifier.step,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              state.isCompiling
                  ? '编译中...'
                  : state.isRunning
                      ? state.isStepMode
                          ? '调试中 (第 ${state.currentLine} 行)'
                          : '运行中'
                      : '等待执行',
              style: TextStyle(fontSize: 13, color: Theme.of(context).textTheme.bodyMedium?.color),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          // 执行速度滑块
          if (state.isStepMode)
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.speed, size: 14, color: Colors.grey),
                const SizedBox(width: 4),
                SizedBox(
                  width: 80,
                  child: SliderTheme(
                    data: SliderTheme.of(context).copyWith(
                      trackHeight: 2,
                      thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6),
                      overlayShape: SliderComponentShape.noOverlay,
                    ),
                    child: Slider(
                      value: state.executionSpeed.toDouble(),
                      min: 0,
                      max: 500,
                      divisions: 10,
                      label: '${state.executionSpeed}ms',
                      onChanged: (v) => notifier.setExecutionSpeed(v.toInt()),
                    ),
                  ),
                ),
              ],
            ),
          ToolButton(
            icon: isDark ? Icons.light_mode : Icons.dark_mode,
            onPressed: onToggleTheme,
          ),
          ToolButton(
            icon: Icons.help_outline,
            onPressed: notifier.showIntro,
          ),
          if (state.output.isNotEmpty)
            ToolButton(
              icon: Icons.delete_outline,
              onPressed: notifier.clearOutput,
            ),
        ],
      ),
    );
  }
}
