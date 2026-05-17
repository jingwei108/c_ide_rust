import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/unified_state.dart';
import '../providers/unified_provider.dart';

class ExecutionControlPanel extends ConsumerWidget {
  final VoidCallback onRun;

  const ExecutionControlPanel({
    super.key,
    required this.onRun,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(unifiedProvider);
    final controller = ref.read(unifiedProvider.notifier);

    if (state.phase == ExecutionPhase.idle) {
      return const SizedBox.shrink();
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 运行时异常提示条
        if (state.trapMessage != null)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            color: Colors.red.shade900,
            child: Row(
              children: [
                const Icon(Icons.warning_amber, color: Colors.white, size: 16),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    state.trapMessage!,
                    style: const TextStyle(color: Colors.white, fontSize: 12),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                TextButton(
                  onPressed: () {
                    ref.read(unifiedProvider.notifier).onCodeChanged();
                  },
                  style: TextButton.styleFrom(
                    foregroundColor: Colors.white,
                    padding: EdgeInsets.zero,
                    minimumSize: const Size(0, 0),
                    tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  ),
                  child: const Text('重置', style: TextStyle(fontSize: 12)),
                ),
              ],
            ),
          ),
        Container(
          height: 48,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            border: Border(
              top: BorderSide(color: Theme.of(context).dividerColor),
            ),
          ),
          child: Row(
        children: [
          // 播放/暂停按钮
          _buildPlayPauseButton(state, controller, onRun),
          // 单步按钮
          if (state.canStep)
            IconButton(
              icon: const Icon(Icons.skip_next, size: 20),
              tooltip: '单步',
              onPressed: controller.stepNext,
            ),
          // 进度条
          if (state.showSlider)
            Expanded(
              child: Slider(
                min: 0,
                max: math.max(state.maxCollectedStep.toDouble(), 1),
                value: state.currentStep.clamp(0, state.maxCollectedStep).toDouble(),
                divisions: state.maxCollectedStep > 0 ? state.maxCollectedStep : null,
                label: _buildSliderLabel(state),
                onChangeStart: (_) => controller.pause(),
                onChanged: (v) => controller.onSliderChanged(v.round()),
                onChangeEnd: (v) => controller.seekTo(v.round()),
              ),
            ),
          // 播放速度
          if (state.phase == ExecutionPhase.collecting ||
              state.phase == ExecutionPhase.paused ||
              state.phase == ExecutionPhase.stepMode)
            _buildSpeedButton(state, controller),
          // 步数显示
          Padding(
            padding: const EdgeInsets.only(left: 8),
            child: Text(
              '${state.currentStep} / ${state.maxCollectedStep}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        ],
      ),
    ),
  ],
);
  }

  String _buildSliderLabel(UnifiedState state) {
    if (state.currentStep >= 0 && state.currentStep < state.frameCache.length) {
      final payload = state.frameCache[state.currentStep];
      if (payload.semanticLabel.isNotEmpty) {
        return payload.semanticLabel;
      }
    }
    return '第 ${state.currentStep} 步';
  }

  Widget _buildPlayPauseButton(
    UnifiedState state,
    UnifiedNotifier controller,
    VoidCallback onRun,
  ) {
    IconData icon;
    VoidCallback? onPressed;
    String tooltip;

    switch (state.phase) {
      case ExecutionPhase.idle:
        return const SizedBox.shrink();
      case ExecutionPhase.compiling:
        icon = Icons.hourglass_top;
        onPressed = null;
        tooltip = '编译中';
      case ExecutionPhase.collecting:
        icon = Icons.pause;
        onPressed = controller.pause;
        tooltip = '暂停';
      case ExecutionPhase.paused:
      case ExecutionPhase.stepMode:
        icon = Icons.play_arrow;
        onPressed = controller.resume;
        tooltip = '继续';
      case ExecutionPhase.playback:
        icon = Icons.play_arrow;
        onPressed = controller.resume;
        tooltip = '从当前步继续执行';
      case ExecutionPhase.seeking:
        icon = Icons.hourglass_top;
        onPressed = null;
        tooltip = '加载中';
      case ExecutionPhase.error:
        icon = Icons.play_arrow;
        onPressed = onRun;
        tooltip = '重新运行';
    }

    return IconButton(
      icon: Icon(icon, size: 24),
      tooltip: tooltip,
      onPressed: onPressed,
    );
  }

  Widget _buildSpeedButton(UnifiedState state, UnifiedNotifier controller) {
    return PopupMenuButton<double>(
      tooltip: '播放速度',
      initialValue: state.playbackSpeed,
      onSelected: controller.setPlaybackSpeed,
      itemBuilder: (context) => [
        const PopupMenuItem(value: 0.5, child: Text('0.5x')),
        const PopupMenuItem(value: 1.0, child: Text('1.0x')),
        const PopupMenuItem(value: 2.0, child: Text('2.0x')),
        const PopupMenuItem(value: 4.0, child: Text('4.0x')),
      ],
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: Text(
          '${state.playbackSpeed}x',
          style: const TextStyle(fontSize: 12),
        ),
      ),
    );
  }
}
